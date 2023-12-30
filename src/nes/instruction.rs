use arrayvec::ArrayVec;

use super::{bus::Bus, cpu::CPU};

pub struct DecodedInstruction {
    pub(crate) byte_sequence: ArrayVec<u8, 3>,
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
                    byte_sequence: self.byte_sequence_absolute(opcode, bus)?,
                    cycles_total: 3,
                    cycles_remaining: 3,
                };

                self.registers.program_counter =
                    CPU::operand_absolute(&instr.byte_sequence) as usize;

                Ok(instr)
            }

            _ => Err("Invalid opcode"),
        }
    }

    fn operand_absolute(byte_sequence: &[u8]) -> u16 {
        u16::from_be_bytes(byte_sequence[1..].try_into().unwrap())
    }

    fn byte_sequence_absolute<'a, T: Bus>(
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
