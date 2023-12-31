use std::fmt::Display;
use std::io::Write;

use bitfield::Bit;
use tock_registers::{
    fields::Field,
    interfaces::{ReadWriteable, Readable},
};

use super::{
    bus::Bus,
    cpu::{
        Status::{self, Register},
        CPU,
    },
};

pub enum Operand {
    ABSOLUTE(u16),
    IMMEDIATE(u8),
    ZEROPAGE { operand: u8, old_byte: u8 },
    RELATIVE { operand: u8, program_counter: usize },
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::ABSOLUTE(val) => write!(f, "${:02X}", val),
            Operand::RELATIVE {
                operand,
                program_counter,
            } => write!(f, "${:02X}", *operand as usize + program_counter),
            Operand::IMMEDIATE(val) => write!(f, "#${:02X}", val),
            Operand::ZEROPAGE { operand, old_byte } => {
                write!(f, "${:02X} = {:02X}", operand, old_byte)
            }
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
            0x18 => {
                write!(self.log_file, "CLC").unwrap();
                Ok(self.clc())
            }
            0x20 => {
                let operand = self.get_operand_absolute(bus)?;
                write!(self.log_file, "JSR {}", operand).unwrap();
                Ok(self.jsr(operand, bus)?)
            }
            0x38 => {
                write!(self.log_file, "SEC").unwrap();
                Ok(self.sec())
            }
            0x4C => {
                let operand = self.get_operand_absolute(bus)?;
                write!(self.log_file, "JMP {}", operand).unwrap();
                Ok(self.jmp(operand)?)
            }
            0x85 => {
                let operand = self.get_operand_zeropage(bus)?;
                write!(self.log_file, "STA {}", operand).unwrap();
                Ok(self.sta(operand, bus)?)
            }
            0x86 => {
                let operand = self.get_operand_zeropage(bus)?;
                write!(self.log_file, "STX {}", operand).unwrap();
                Ok(self.stx(operand, bus)?)
            }
            0x90 => {
                let operand = self.get_operand_relative(bus)?;
                write!(self.log_file, "BCC {}", operand).unwrap();
                Ok(self.bcc(operand)?)
            }
            0xA2 => {
                let operand = self.get_operand_immediate(bus)?;
                write!(self.log_file, "LDX {}", operand).unwrap();
                Ok(self.ldx(operand)?)
            }
            0xA9 => {
                let operand = self.get_operand_immediate(bus)?;
                write!(self.log_file, "LDA {}", operand).unwrap();
                Ok(self.lda(operand)?)
            }
            0xB0 => {
                let operand = self.get_operand_relative(bus)?;
                write!(self.log_file, "BCS {}", operand).unwrap();
                Ok(self.bcs(operand)?)
            }
            0xD0 => {
                let operand = self.get_operand_relative(bus)?;
                write!(self.log_file, "BNE {}", operand).unwrap();
                Ok(self.bne(operand)?)
            }
            0xEA => {
                write!(self.log_file, "NOP").unwrap();
                Ok(2)
            }
            0xF0 => {
                let operand = self.get_operand_relative(bus)?;
                write!(self.log_file, "BEQ {}", operand).unwrap();
                Ok(self.beq(operand)?)
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

    fn get_operand_relative<T: Bus>(&mut self, bus: &mut T) -> Result<Operand, &'static str> {
        let byte = bus.read_byte(self.registers.program_counter)?;
        self.registers.program_counter += 1;
        write!(self.log_file, "{:02X}    ", byte).unwrap();
        Ok(Operand::RELATIVE {
            operand: byte,
            program_counter: self.registers.program_counter,
        })
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
                self.set_zero_flag_if(val == 0);
                self.set_neg_flag_if(val.bit(7));
                Ok(2)
            }
            _ => Err("Unsupported instruction occured"),
        }
    }

    fn lda(&mut self, operand: Operand) -> Result<u8, &'static str> {
        match operand {
            Operand::IMMEDIATE(val) => {
                self.registers.accumulator = val;
                self.set_zero_flag_if(val == 0);
                self.set_neg_flag_if(val.bit(7));
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

    fn sta<T: Bus>(&mut self, operand: Operand, bus: &mut T) -> Result<u8, &'static str> {
        match operand {
            Operand::ZEROPAGE { operand, .. } => {
                bus.write_byte(operand as usize, self.registers.accumulator)?;
                Ok(3)
            }
            _ => Err("Unsupported instruction occured"),
        }
    }

    fn sec(&mut self) -> u8 {
        self.registers.status_register.modify(Status::CARRY::SET);
        2
    }

    fn clc(&mut self) -> u8 {
        self.registers.status_register.modify(Status::CARRY::CLEAR);
        2
    }

    fn bcs(&mut self, operand: Operand) -> Result<u8, &'static str> {
        self.branchif(operand, true, Status::CARRY)
    }

    fn bcc(&mut self, operand: Operand) -> Result<u8, &'static str> {
        self.branchif(operand, false, Status::CARRY)
    }

    fn beq(&mut self, operand: Operand) -> Result<u8, &'static str> {
        self.branchif(operand, true, Status::ZERO)
    }

    fn bne(&mut self, operand: Operand) -> Result<u8, &'static str> {
        self.branchif(operand, false, Status::ZERO)
    }

    fn branchif(
        &mut self,
        operand: Operand,
        set: bool,
        flag: Field<u8, Register>,
    ) -> Result<u8, &'static str> {
        match operand {
            Operand::RELATIVE {
                operand,
                program_counter,
            } => {
                let check = if set {
                    self.registers.status_register.is_set(flag)
                } else {
                    !self.registers.status_register.is_set(flag)
                };

                let mut cycle_count = 2;
                if check {
                    cycle_count += 1;
                    let new_addr = operand as usize + program_counter;

                    if CPU::will_cross_boundary(new_addr, self.registers.program_counter) {
                        cycle_count += 1;
                    }

                    self.registers.program_counter = new_addr;
                }

                Ok(cycle_count)
            }
            _ => Err("Unsupported instruction occured"),
        }
    }
}
