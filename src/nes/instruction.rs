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
    IMMEDIATE,
    ZEROPAGE,
    IMPLIED,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::ABSOLUTE(val) => write!(f, "${:02X}", val),
            Operand::IMMEDIATE => todo!(),
            Operand::ZEROPAGE => todo!(),
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
            0x4C => {
                let operand = self.get_operand_absolute(bus)?;
                write!(self.log_file, "JMP {}", operand).unwrap();
                Ok(self.jmp(operand)?)
            }
            0xEA => {
                write!(self.log_file, "NOP").unwrap();
                Ok(2)
            }
            _ => Err("Invalid opcode"),
        }
    }

    fn get_operand_absolute<T: Bus>(&mut self, bus: &mut T) -> Result<Operand, &'static str> {
        let mut bytes = [0u8; 2];
        bus.read_exact(self.registers.program_counter, &mut bytes)?;
        write!(self.log_file, "{:02X} {:02X} ", bytes[0], bytes[1]).unwrap();
        Ok(Operand::ABSOLUTE(u16::from_le_bytes(bytes)))
    }

    fn jmp(&mut self, operand: Operand) -> Result<u8, &'static str> {
        match operand {
            Operand::ABSOLUTE(addr) => {
                self.registers.program_counter = addr as usize;
                Ok(3)
            }
            _ => Err(""),
        }
    }
}
