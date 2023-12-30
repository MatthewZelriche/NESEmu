use std::fmt::Display;
use std::io::Write;

use arrayvec::ArrayVec;
use bitfield::Bit;
use eframe::egui::os::OperatingSystem;
use tock_registers::interfaces::ReadWriteable;

use super::{
    bus::Bus,
    cpu::{Status, CPU},
};

pub enum Operand {
    ABSOLUTE(u16),
    IMMEDIATE(u8),
    ZEROPAGE { operand: u8, old_byte: u8 },
    IMPLIED,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::ABSOLUTE(val) => write!(f, "${:02X}", val),
            Operand::IMMEDIATE(val) => write!(f, "#${:02X}", val),
            Operand::ZEROPAGE { operand, old_byte } => {
                write!(f, "${:02X} = {:02X}", operand, old_byte)
            }
            Operand::IMPLIED => todo!(),
        }
    }
}

impl CPU {
    pub fn execute_opcode<'a, T: Bus>(
        &'a mut self,
        opcode: u8,
        bus: &'a mut T,
    ) -> Result<u8, &'static str> {
        write!(self.log_file, "{:X} ", opcode).unwrap();

        match opcode {
            0x20 => {
                let operand = self.get_operand_absolute(bus)?;
                write!(self.log_file, "JSR {}", operand).unwrap();
                Ok(self.jsr(operand, bus)?)
            }
            0x4C => {
                let operand = self.get_operand_absolute(bus)?;
                write!(self.log_file, "JMP {}", operand).unwrap();
                Ok(self.jmp(operand)?)
            }
            0x86 => {
                let operand = self.get_operand_zeropage(bus)?;
                write!(self.log_file, "STX {}", operand).unwrap();
                Ok(self.stx(operand, bus)?)
            }
            0xA2 => {
                let operand = self.get_operand_immediate(bus)?;
                write!(self.log_file, "LDX {}", operand).unwrap();
                Ok(self.ldx(operand)?)
            }
            0xEA => {
                write!(self.log_file, "NOP").unwrap();
                Ok(2)
            }
            _ => Err("Invalid opcode"),
        }
    }

    fn get_operand_immediate<T: Bus>(&mut self, bus: &mut T) -> Result<Operand, &'static str> {
        let byte = bus.read_byte(self.registers.program_counter)?;
        self.registers.program_counter += 1;
        write!(self.log_file, "{:02X}    ", byte).unwrap();
        Ok(Operand::IMMEDIATE(byte))
    }

    fn get_operand_zeropage<T: Bus>(&mut self, bus: &mut T) -> Result<Operand, &'static str> {
        let operand = bus.read_byte(self.registers.program_counter)?;
        self.registers.program_counter += 1;
        write!(self.log_file, "{:02X}    ", operand).unwrap();
        let old_byte = bus.read_byte(operand as usize)?;
        Ok(Operand::ZEROPAGE { operand, old_byte })
    }

    fn get_operand_absolute<T: Bus>(&mut self, bus: &mut T) -> Result<Operand, &'static str> {
        let mut bytes = [0u8; 2];
        bus.read_exact(self.registers.program_counter, &mut bytes)?;
        self.registers.program_counter += 2;
        write!(self.log_file, "{:02X} {:02X} ", bytes[0], bytes[1]).unwrap();
        Ok(Operand::ABSOLUTE(u16::from_le_bytes(bytes)))
    }

    fn jmp(&mut self, operand: Operand) -> Result<u8, &'static str> {
        match operand {
            Operand::ABSOLUTE(addr) => {
                self.registers.program_counter = addr as usize;
                Ok(3)
            }
            _ => Err("Unsupported instruction occured"),
        }
    }

    fn jsr<T: Bus>(&mut self, operand: Operand, bus: &mut T) -> Result<u8, &'static str> {
        match operand {
            Operand::ABSOLUTE(addr) => {
                self.push_stack(&u16::to_le_bytes(addr), bus)?;
                self.registers.program_counter = addr as usize;
                Ok(6)
            }
            _ => Err("Unsupported instruction occured"),
        }
    }

    fn ldx(&mut self, operand: Operand) -> Result<u8, &'static str> {
        match operand {
            Operand::IMMEDIATE(val) => {
                self.registers.x_reg = val;
                if val == 0 {
                    self.registers.status_register.modify(Status::ZERO::SET);
                }
                if val.bit(7) {
                    self.registers.status_register.modify(Status::NEGATIVE::SET);
                }
                Ok(2)
            }
            _ => Err("Unsupported instruction occured"),
        }
    }

    fn stx<T: Bus>(&mut self, operand: Operand, bus: &mut T) -> Result<u8, &'static str> {
        match operand {
            Operand::ZEROPAGE { operand, .. } => {
                bus.write_byte(operand as usize, self.registers.x_reg)?;
                Ok(3)
            }
            _ => Err("Unsupported instruction occured"),
        }
    }
}
