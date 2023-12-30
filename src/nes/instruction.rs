use std::fmt::Display;

use arrayvec::ArrayVec;
use bitfield::Bit;
use tock_registers::interfaces::ReadWriteable;

use super::{
    bus::Bus,
    cpu::{Status, CPU},
};

pub enum AddressingMode {
    ABSOLUTE,
    IMMEDIATE,
}

pub struct DecodedInstruction {
    pub(crate) byte_sequence: ArrayVec<u8, 3>,
    pub(crate) address_mode: AddressingMode,
    pub(crate) mnemonic: &'static str,
    pub(crate) cycles_total: u8,
    pub(crate) cycles_remaining: u8,
}

impl CPU {
    pub fn execute_opcode<'a, T: Bus>(
        &'a mut self,
        opcode: u8,
        bus: &'a mut T,
    ) -> Result<DecodedInstruction, &'static str> {
        match opcode {
            0x4C => {
                let instr = DecodedInstruction {
                    mnemonic: "JMP",
                    address_mode: AddressingMode::ABSOLUTE,
                    byte_sequence: self.byte_sequence_absolute(opcode, bus)?,
                    cycles_total: 3,
                    cycles_remaining: 3,
                };

                self.registers.program_counter =
                    CPU::operand_absolute(&instr.byte_sequence) as usize;

                Ok(instr)
            }
            0xA2 => {
                let instr = DecodedInstruction {
                    mnemonic: "LDX",
                    address_mode: AddressingMode::IMMEDIATE,
                    byte_sequence: self.byte_sequence_immediate(opcode, bus)?,
                    cycles_total: 2,
                    cycles_remaining: 2,
                };

                self.registers.x_reg = instr.byte_sequence[1];
                if self.registers.x_reg == 0 {
                    self.registers.status_register.modify(Status::ZERO::SET);
                }
                if self.registers.x_reg.bit(7) {
                    self.registers.status_register.modify(Status::NEGATIVE::SET);
                }

                Ok(instr)
            }

            _ => Err("Invalid opcode"),
        }
    }

    fn operand_absolute(byte_sequence: &[u8]) -> u16 {
        u16::from_le_bytes(byte_sequence[1..].try_into().unwrap())
    }

    fn byte_sequence_immediate<T: Bus>(
        &mut self,
        opcode: u8,
        bus: &mut T,
    ) -> Result<ArrayVec<u8, 3>, &'static str> {
        let addr = self.registers.program_counter;
        Ok(ArrayVec::from([opcode, bus.read_byte(addr)?, 0x0]))
    }

    fn byte_sequence_absolute<T: Bus>(
        &mut self,
        opcode: u8,
        bus: &mut T,
    ) -> Result<ArrayVec<u8, 3>, &'static str> {
        let mut byte_sequence = [opcode, 0x0, 0x0];
        let addr = self.registers.program_counter;
        bus.read_exact(addr, &mut byte_sequence[1..])?;
        Ok(ArrayVec::from(byte_sequence))
    }
}

impl Display for DecodedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.address_mode {
            AddressingMode::ABSOLUTE => {
                write!(
                    f,
                    "{:X} {:X} {:X}  {} ${:X}",
                    self.byte_sequence[0],
                    self.byte_sequence[1],
                    self.byte_sequence[2],
                    self.mnemonic,
                    CPU::operand_absolute(&self.byte_sequence)
                )
            }
            AddressingMode::IMMEDIATE => {
                write!(
                    f,
                    "{:X} {:X} {} #${:X}",
                    self.byte_sequence[0],
                    self.byte_sequence[1],
                    self.mnemonic,
                    self.byte_sequence[1]
                )
            }
        }
    }
}
