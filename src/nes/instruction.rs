use std::io::Write;

use bitfield::{Bit, BitMut};
use tock_registers::{
    fields::Field,
    interfaces::{ReadWriteable, Readable},
};

use super::{
    bus::{Bus, BusImpl},
    cpu::{
        Status::{self, Register},
        CPU,
    },
};

pub enum AddressMode {
    IMPLIED,
    IMMEDIATE,
    ABSOLUTE(bool),
    RELATIVE,
    ZEROPAGE,
    ACCUMULATOR,
    INDIRECTX,
    INDIRECTY,
}

pub struct Opcode {
    mnemonic: &'static str,
    mode: AddressMode,
    bytes: [u8; 3],
    num_bytes: u8,
    cycles: u8,
    execute: fn(&mut CPU, usize, AddressMode, u8, &mut BusImpl) -> Result<u8, &'static str>,
}

impl CPU {
    pub fn lookup_opcode(&mut self, opcode: u8, bus: &mut BusImpl) -> Result<Opcode, &'static str> {
        match opcode {
            0x01 => Ok(Opcode {
                mnemonic: "ORA",
                mode: AddressMode::INDIRECTX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ora,
            }),
            0x05 => Ok(Opcode {
                mnemonic: "ORA",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ora,
            }),
            0x18 => Ok(Opcode {
                mnemonic: "CLC",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::clc,
            }),
            0x06 => Ok(Opcode {
                mnemonic: "ASL",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::asl,
            }),
            0x08 => Ok(Opcode {
                mnemonic: "PHP",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 3,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::php,
            }),
            0x09 => Ok(Opcode {
                mnemonic: "ORA",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ora,
            }),
            0x0A => Ok(Opcode {
                mnemonic: "ASL",
                mode: AddressMode::ACCUMULATOR,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::asl,
            }),
            0x0D => Ok(Opcode {
                mnemonic: "ORA",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ora,
            }),
            0x0E => Ok(Opcode {
                mnemonic: "ASL",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 6,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::asl,
            }),
            0x10 => Ok(Opcode {
                mnemonic: "BPL",
                mode: AddressMode::RELATIVE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::bpl,
            }),
            0x20 => Ok(Opcode {
                mnemonic: "JSR",
                mode: AddressMode::ABSOLUTE(false),
                num_bytes: 3,
                cycles: 6,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::jsr,
            }),
            0x21 => Ok(Opcode {
                mnemonic: "AND",
                mode: AddressMode::INDIRECTX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::and,
            }),
            0x24 => Ok(Opcode {
                mnemonic: "BIT",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::bit,
            }),
            0x25 => Ok(Opcode {
                mnemonic: "AND",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::and,
            }),
            0x26 => Ok(Opcode {
                mnemonic: "ROL",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::rol,
            }),
            0x28 => Ok(Opcode {
                mnemonic: "PLP",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 4,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::plp,
            }),
            0x2A => Ok(Opcode {
                mnemonic: "ROL",
                mode: AddressMode::ACCUMULATOR,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::rol,
            }),
            0x29 => Ok(Opcode {
                mnemonic: "AND",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::and,
            }),
            0x2C => Ok(Opcode {
                mnemonic: "BIT",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::bit,
            }),
            0x2D => Ok(Opcode {
                mnemonic: "AND",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::and,
            }),
            0x2E => Ok(Opcode {
                mnemonic: "ROL",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 6,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::rol,
            }),
            0x30 => Ok(Opcode {
                mnemonic: "BMI",
                mode: AddressMode::RELATIVE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::bmi,
            }),
            0x38 => Ok(Opcode {
                mnemonic: "SEC",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::sec,
            }),
            0x40 => Ok(Opcode {
                mnemonic: "RTI",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 6,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::rti,
            }),
            0x41 => Ok(Opcode {
                mnemonic: "EOR",
                mode: AddressMode::INDIRECTX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::eor,
            }),
            0x45 => Ok(Opcode {
                mnemonic: "EOR",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::eor,
            }),
            0x46 => Ok(Opcode {
                mnemonic: "LSR",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::lsr,
            }),
            0x49 => Ok(Opcode {
                mnemonic: "EOR",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::eor,
            }),
            0x4A => Ok(Opcode {
                mnemonic: "LSR",
                mode: AddressMode::ACCUMULATOR,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::lsr,
            }),
            0x4C => Ok(Opcode {
                mnemonic: "JMP",
                mode: AddressMode::ABSOLUTE(false),
                num_bytes: 3,
                cycles: 3,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::jmp,
            }),
            0x4D => Ok(Opcode {
                mnemonic: "EOR",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::eor,
            }),
            0x4E => Ok(Opcode {
                mnemonic: "LSR",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 6,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::lsr,
            }),
            0x48 => Ok(Opcode {
                mnemonic: "PHA",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 3,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::pha,
            }),
            0x50 => Ok(Opcode {
                mnemonic: "BVC",
                mode: AddressMode::RELATIVE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::bvc,
            }),
            0x60 => Ok(Opcode {
                mnemonic: "RTS",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 6,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::rts,
            }),
            0x61 => Ok(Opcode {
                mnemonic: "ADC",
                mode: AddressMode::INDIRECTX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::adc,
            }),
            0x65 => Ok(Opcode {
                mnemonic: "ADC",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::adc,
            }),
            0x66 => Ok(Opcode {
                mnemonic: "ROR",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ror,
            }),
            0x68 => Ok(Opcode {
                mnemonic: "PLA",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 4,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::pla,
            }),
            0x69 => Ok(Opcode {
                mnemonic: "ADC",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::adc,
            }),
            0x6A => Ok(Opcode {
                mnemonic: "ROR",
                mode: AddressMode::ACCUMULATOR,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::ror,
            }),
            0x6D => Ok(Opcode {
                mnemonic: "ADC",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::adc,
            }),
            0x6E => Ok(Opcode {
                mnemonic: "ROR",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 6,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ror,
            }),
            0x70 => Ok(Opcode {
                mnemonic: "BVS",
                mode: AddressMode::RELATIVE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::bvs,
            }),
            0x78 => Ok(Opcode {
                mnemonic: "SEI",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::sei,
            }),
            0x81 => Ok(Opcode {
                mnemonic: "STA",
                mode: AddressMode::INDIRECTX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sta,
            }),
            0x84 => Ok(Opcode {
                mnemonic: "STY",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sty,
            }),
            0x85 => Ok(Opcode {
                mnemonic: "STA",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sta,
            }),
            0x86 => Ok(Opcode {
                mnemonic: "STX",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::stx,
            }),
            0x88 => Ok(Opcode {
                mnemonic: "DEY",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::dey,
            }),
            0x8A => Ok(Opcode {
                mnemonic: "TXA",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::txa,
            }),
            0x8C => Ok(Opcode {
                mnemonic: "STY",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::sty,
            }),
            0x8D => Ok(Opcode {
                mnemonic: "STA",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::sta,
            }),
            0x8E => Ok(Opcode {
                mnemonic: "STX",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::stx,
            }),
            0x90 => Ok(Opcode {
                mnemonic: "BCC",
                mode: AddressMode::RELATIVE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::bcc,
            }),
            0x98 => Ok(Opcode {
                mnemonic: "TYA",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::tya,
            }),
            0x9A => Ok(Opcode {
                mnemonic: "TXS",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::txs,
            }),
            0xA0 => Ok(Opcode {
                mnemonic: "LDY",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ldy,
            }),
            0xA1 => Ok(Opcode {
                mnemonic: "LDA",
                mode: AddressMode::INDIRECTX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::lda,
            }),
            0xA2 => Ok(Opcode {
                mnemonic: "LDX",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ldx,
            }),
            0xA4 => Ok(Opcode {
                mnemonic: "LDY",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ldy,
            }),
            0xA5 => Ok(Opcode {
                mnemonic: "LDA",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::lda,
            }),
            0xA6 => Ok(Opcode {
                mnemonic: "LDX",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ldx,
            }),
            0xA8 => Ok(Opcode {
                mnemonic: "TAY",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::tay,
            }),
            0xA9 => Ok(Opcode {
                mnemonic: "LDA",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::lda,
            }),
            0xAA => Ok(Opcode {
                mnemonic: "TAX",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::tax,
            }),
            0xAC => Ok(Opcode {
                mnemonic: "LDY",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ldy,
            }),
            0xAD => Ok(Opcode {
                mnemonic: "LDA",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::lda,
            }),
            0xAE => Ok(Opcode {
                mnemonic: "LDX",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ldx,
            }),
            0xB0 => Ok(Opcode {
                mnemonic: "BCS",
                mode: AddressMode::RELATIVE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::bcs,
            }),
            0xB1 => Ok(Opcode {
                mnemonic: "LDA",
                mode: AddressMode::INDIRECTY,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::lda,
            }),
            0xB8 => Ok(Opcode {
                mnemonic: "CLV",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::clv,
            }),
            0xBA => Ok(Opcode {
                mnemonic: "TSX",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::tsx,
            }),
            0xC0 => Ok(Opcode {
                mnemonic: "CPY",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cpy,
            }),
            0xC1 => Ok(Opcode {
                mnemonic: "CMP",
                mode: AddressMode::INDIRECTX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cmp,
            }),
            0xC4 => Ok(Opcode {
                mnemonic: "CPY",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cpy,
            }),
            0xC5 => Ok(Opcode {
                mnemonic: "CMP",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cmp,
            }),
            0xC6 => Ok(Opcode {
                mnemonic: "DEC",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::dec,
            }),
            0xC8 => Ok(Opcode {
                mnemonic: "INY",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::iny,
            }),
            0xC9 => Ok(Opcode {
                mnemonic: "CMP",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cmp,
            }),
            0xCA => Ok(Opcode {
                mnemonic: "DEX",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::dex,
            }),
            0xCC => Ok(Opcode {
                mnemonic: "CPY",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::cpy,
            }),
            0xCD => Ok(Opcode {
                mnemonic: "CMP",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::cmp,
            }),
            0xCE => Ok(Opcode {
                mnemonic: "DEC",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 6,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::dec,
            }),
            0xD0 => Ok(Opcode {
                mnemonic: "BNE",
                mode: AddressMode::RELATIVE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::bne,
            }),
            0xD8 => Ok(Opcode {
                mnemonic: "CLD",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::cld,
            }),
            0xE0 => Ok(Opcode {
                mnemonic: "CPX",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cpx,
            }),
            0xE1 => Ok(Opcode {
                mnemonic: "SBC",
                mode: AddressMode::INDIRECTX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sbc,
            }),
            0xE4 => Ok(Opcode {
                mnemonic: "CPX",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cpx,
            }),
            0xE5 => Ok(Opcode {
                mnemonic: "SBC",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 3,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sbc,
            }),
            0xE6 => Ok(Opcode {
                mnemonic: "INC",
                mode: AddressMode::ZEROPAGE,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::inc,
            }),
            0xE8 => Ok(Opcode {
                mnemonic: "INX",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::inx,
            }),
            0xE9 => Ok(Opcode {
                mnemonic: "SBC",
                mode: AddressMode::IMMEDIATE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sbc,
            }),
            0xEA => Ok(Opcode {
                mnemonic: "NOP",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::nop,
            }),
            0xEC => Ok(Opcode {
                mnemonic: "CPX",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::cpx,
            }),
            0xED => Ok(Opcode {
                mnemonic: "SBC",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::sbc,
            }),
            0xEE => Ok(Opcode {
                mnemonic: "INC",
                mode: AddressMode::ABSOLUTE(true),
                num_bytes: 3,
                cycles: 6,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::inc,
            }),
            0xF0 => Ok(Opcode {
                mnemonic: "BEQ",
                mode: AddressMode::RELATIVE,
                num_bytes: 2,
                cycles: 2,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::beq,
            }),
            0xF8 => Ok(Opcode {
                mnemonic: "SED",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::sed,
            }),
            _ => Err("Invalid opcode"),
        }
    }

    fn write_opcode(&mut self, opcode: &Opcode, bus: &mut BusImpl) {
        let mut fmt_string = String::new();

        if opcode.num_bytes == 1 {
            fmt_string = format!("{:02X}{:<8}{} ", opcode.bytes[0], "", opcode.mnemonic);

            match opcode.mode {
                AddressMode::ACCUMULATOR => {
                    fmt_string = format!("{}A ", fmt_string);
                }
                _ => {}
            }
        } else if opcode.num_bytes == 2 {
            fmt_string = format!(
                "{:02X} {:02X}{:<5}{} ",
                opcode.bytes[0], opcode.bytes[1], "", opcode.mnemonic
            );

            match opcode.mode {
                AddressMode::IMMEDIATE => {
                    fmt_string = format!("{}#${:02X}", fmt_string, opcode.bytes[1]);
                }
                AddressMode::RELATIVE => {
                    fmt_string = format!(
                        "{}${:02X}",
                        fmt_string,
                        opcode.bytes[1] as usize + self.registers.program_counter
                    );
                }
                AddressMode::ZEROPAGE => {
                    let address_value = bus.read_byte(opcode.bytes[1] as usize).unwrap();
                    fmt_string = format!(
                        "{}${:02X} = {:02X}",
                        fmt_string, opcode.bytes[1], address_value
                    );
                }
                AddressMode::INDIRECTX => {
                    let lsb_addr = opcode.bytes[1].wrapping_add(self.registers.x_reg);
                    let addr = self.fetch_operand_address(opcode, bus);
                    fmt_string = format!(
                        "{}(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                        fmt_string,
                        opcode.bytes[1],
                        lsb_addr,
                        addr,
                        bus.read_byte(addr).unwrap()
                    );
                }
                AddressMode::INDIRECTY => {
                    let addr = self.fetch_operand_address(opcode, bus);
                    fmt_string = format!(
                        "{}(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                        fmt_string,
                        opcode.bytes[1],
                        self.fetch_indirect_y_base_addr(opcode, bus),
                        addr,
                        bus.read_byte(addr).unwrap()
                    );
                }
                _ => {} // should never happen
            }
        } else if opcode.num_bytes == 3 {
            fmt_string = format!(
                "{:02X} {:02X} {:02X}  {} ",
                opcode.bytes[0], opcode.bytes[1], opcode.bytes[2], opcode.mnemonic
            );

            match opcode.mode {
                AddressMode::ABSOLUTE(mem_modify) => {
                    let operand = u16::from_le_bytes(opcode.bytes[1..].try_into().unwrap());
                    fmt_string = format!("{}${:04X}", fmt_string, operand);

                    if mem_modify {
                        let byte = bus.read_byte(operand as usize).unwrap();
                        fmt_string = format!("{} = {:02X}", fmt_string, byte);
                    }
                }
                _ => {} // should never happen
            }
        }

        fmt_string = format!("{:<42}", fmt_string);
        write!(self.log_file, "{}", fmt_string).unwrap();
    }

    pub fn fetch_indirect_y_base_addr(&self, opcode: &Opcode, bus: &mut BusImpl) -> usize {
        let lsb_addr = opcode.bytes[1] as usize;
        let msb_addr = (lsb_addr as u8).wrapping_add(1) as usize;
        let addr_bytes = [
            bus.read_byte(lsb_addr).unwrap(),
            bus.read_byte(msb_addr).unwrap(),
        ];
        u16::from_le_bytes(addr_bytes) as usize
    }

    pub fn execute_opcode<'a>(
        &'a mut self,
        opcode_val: u8,
        bus: &'a mut BusImpl,
    ) -> Result<u8, &'static str> {
        let mut opcode = self.lookup_opcode(opcode_val, bus)?;
        self.write_opcode(&opcode, bus);

        let addr = self.fetch_operand_address(&opcode, bus);

        match opcode.mode {
            AddressMode::INDIRECTY => {
                let base_addr = self.fetch_indirect_y_base_addr(&opcode, bus);
                if CPU::will_cross_boundary(base_addr, addr) {
                    opcode.cycles += 1;
                }
            }
            _ => {} // No cycle adjustment
        }

        (opcode.execute)(self, addr, opcode.mode, opcode.cycles, bus)
    }

    fn fetch_zero_more_bytes(&mut self, opcode: u8) -> [u8; 3] {
        [opcode, 0x0, 0x0]
    }

    fn fetch_one_more_bytes(
        &mut self,
        opcode: u8,
        bus: &mut BusImpl,
    ) -> Result<[u8; 3], &'static str> {
        let bytes = [opcode, bus.read_byte(self.registers.program_counter)?, 0x0];
        self.registers.program_counter += 1;
        Ok(bytes)
    }

    fn fetch_two_more_bytes(
        &mut self,
        opcode: u8,
        bus: &mut BusImpl,
    ) -> Result<[u8; 3], &'static str> {
        let mut bytes = [opcode, 0x0, 0x0];
        bus.read_exact(self.registers.program_counter, &mut bytes[1..])?;
        self.registers.program_counter += 2;
        Ok(bytes)
    }

    fn fetch_operand_address(&mut self, opcode: &Opcode, bus: &mut BusImpl) -> usize {
        match opcode.mode {
            AddressMode::IMPLIED | AddressMode::ACCUMULATOR => 0x0, // Address is irrelevant for implied and ACC
            AddressMode::IMMEDIATE => self.registers.program_counter - 1,
            AddressMode::ABSOLUTE(_) => {
                u16::from_le_bytes(opcode.bytes[1..].try_into().unwrap()) as usize
            }
            AddressMode::RELATIVE => opcode.bytes[1] as usize + self.registers.program_counter,
            AddressMode::ZEROPAGE => opcode.bytes[1] as usize,
            AddressMode::INDIRECTX => {
                // Indirect zeropage is tricky, because if we are given a lsb of 0xFF,
                // we need to discover the high byte at 0x00, not 0x100
                let lsb_addr = opcode.bytes[1].wrapping_add(self.registers.x_reg) as usize;
                let msb_addr = (lsb_addr as u8).wrapping_add(1) as usize;
                let addr_bytes = [
                    bus.read_byte(lsb_addr).unwrap(),
                    bus.read_byte(msb_addr).unwrap(),
                ];
                u16::from_le_bytes(addr_bytes) as usize
            }
            AddressMode::INDIRECTY => {
                let addr = self.fetch_indirect_y_base_addr(opcode, bus) as u16;
                let addr_indirect = addr.wrapping_add(self.registers.y_reg as u16) as usize;

                addr_indirect
            }
        }
    }

    fn rti(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let mut byte = [0];
        self.pop_stack(&mut byte, bus)?;
        self.set_status_register(byte[0]);

        let mut pc = [0u8; 2];
        self.pop_stack(&mut pc, bus)?;
        self.registers.program_counter = u16::from_le_bytes(pc) as usize;

        Ok(start_cycles)
    }

    fn sbc(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let old_accumulator = self.registers.accumulator;
        let mut mem = bus.read_byte(addr)?;
        mem ^= 0xFF; // Only difference from ADC is that we xor the memory byte thanks to two's complement
        let val16bit: u16 = self.registers.accumulator as u16
            + mem as u16
            + self.registers.status_register.is_set(Status::CARRY) as u16;
        self.registers.accumulator = (val16bit & 0xFF) as u8; // Drop the 8th bit
        self.set_flag_bit_if(0, val16bit.bit(8));
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(
            6,
            ((val16bit ^ old_accumulator as u16) & (val16bit ^ mem as u16)).bit(7),
        );
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));

        Ok(start_cycles)
    }

    fn adc(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let old_accumulator = self.registers.accumulator;
        let mem = bus.read_byte(addr)?;
        let val16bit: u16 = self.registers.accumulator as u16
            + mem as u16
            + self.registers.status_register.is_set(Status::CARRY) as u16;
        self.registers.accumulator = (val16bit & 0xFF) as u8; // Drop the 8th bit
        self.set_flag_bit_if(0, val16bit.bit(8));
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(
            6,
            ((val16bit ^ old_accumulator as u16) & (val16bit ^ mem as u16)).bit(7),
        );
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));

        Ok(start_cycles)
    }

    fn plp(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let mut byte = [0u8];
        self.pop_stack(&mut byte, bus)?;
        self.set_status_register(byte[0]);
        Ok(start_cycles)
    }

    fn pla(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let mut byte = [0u8];
        self.pop_stack(&mut byte, bus)?;
        self.registers.accumulator = byte[0];
        self.set_flag_bit_if(1, byte[0] == 0);
        self.set_flag_bit_if(7, byte[0].bit(7));
        Ok(start_cycles)
    }

    fn php(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        // Instructions that push status flags to the stack always push BFLAG as set
        let mut copy = self.registers.status_register.extract();
        copy.modify(Status::BFLAG::SET);

        let byte = [copy.get()];
        self.push_stack(&byte, bus)?;
        Ok(start_cycles)
    }

    fn pha(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let byte = [self.registers.accumulator];
        self.push_stack(&byte, bus)?;
        Ok(start_cycles)
    }

    fn nop(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        Ok(start_cycles)
    }

    fn clc(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.status_register.modify(Status::CARRY::CLEAR);
        Ok(start_cycles)
    }

    fn sei(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers
            .status_register
            .modify(Status::INT_DISABLE::SET);
        Ok(start_cycles)
    }

    fn sed(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.status_register.modify(Status::DECIMAL::SET);
        Ok(start_cycles)
    }

    fn clv(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers
            .status_register
            .modify(Status::OVERFLOW::CLEAR);
        Ok(start_cycles)
    }

    fn cld(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers
            .status_register
            .modify(Status::DECIMAL::CLEAR);
        Ok(start_cycles)
    }

    fn sec(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.status_register.modify(Status::CARRY::SET);
        Ok(start_cycles)
    }

    fn and(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.accumulator &= bus.read_byte(addr)?;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        Ok(start_cycles)
    }

    fn ora(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.accumulator |= bus.read_byte(addr)?;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        Ok(start_cycles)
    }

    fn eor(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.accumulator ^= bus.read_byte(addr)?;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        Ok(start_cycles)
    }

    fn jsr(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        // Store the current program counter (which, right now, points to the NEXT
        // instruction after the one we are processing)
        // big endian because we need to push to the stack in reverse order of how they should be
        self.push_stack(
            &u16::to_be_bytes((self.registers.program_counter - 1) as u16),
            bus,
        )?;
        self.registers.program_counter = addr;
        Ok(start_cycles)
    }

    fn rts(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let mut addr_bytes = [0u8; 2];
        self.pop_stack(&mut addr_bytes, bus)?;
        self.registers.program_counter = (u16::from_le_bytes(addr_bytes) + 1) as usize;
        Ok(start_cycles)
    }

    fn bit(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let byte = bus.read_byte(addr)?;
        self.set_flag_bit_if(1, self.registers.accumulator & byte == 0);
        self.set_flag_bit_if(6, byte.bit(6));
        self.set_flag_bit_if(7, byte.bit(7));

        Ok(start_cycles)
    }

    fn cmp(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.compare_reg(addr, self.registers.accumulator, start_cycles, bus)
    }

    fn cpy(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.compare_reg(addr, self.registers.y_reg, start_cycles, bus)
    }

    fn cpx(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.compare_reg(addr, self.registers.x_reg, start_cycles, bus)
    }

    fn compare_reg(
        &mut self,
        addr: usize,
        reg_val: u8,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let byte = bus.read_byte(addr)?;
        self.set_flag_bit_if(0, reg_val >= byte);
        self.set_flag_bit_if(1, reg_val == byte);
        self.set_flag_bit_if(7, reg_val.wrapping_sub(byte).bit(7));
        Ok(start_cycles)
    }

    fn tay(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.y_reg = self.registers.accumulator;
        self.set_flag_bit_if(1, self.registers.y_reg == 0);
        self.set_flag_bit_if(7, self.registers.y_reg.bit(7));
        Ok(start_cycles)
    }

    fn tya(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.accumulator = self.registers.y_reg;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        Ok(start_cycles)
    }

    fn tax(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.x_reg = self.registers.accumulator;
        self.set_flag_bit_if(1, self.registers.x_reg == 0);
        self.set_flag_bit_if(7, self.registers.x_reg.bit(7));
        Ok(start_cycles)
    }

    fn txa(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.accumulator = self.registers.x_reg;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        Ok(start_cycles)
    }

    fn tsx(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.x_reg = self.registers.stack_ptr as u8;
        self.set_flag_bit_if(1, self.registers.x_reg == 0);
        self.set_flag_bit_if(7, self.registers.x_reg.bit(7));
        Ok(start_cycles)
    }

    fn txs(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.stack_ptr = self.registers.x_reg as usize;
        Ok(start_cycles)
    }

    fn iny(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.y_reg = self.registers.y_reg.wrapping_add(1);
        self.set_flag_bit_if(1, self.registers.y_reg == 0);
        self.set_flag_bit_if(7, self.registers.y_reg.bit(7));
        Ok(start_cycles)
    }

    fn dey(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.y_reg = self.registers.y_reg.wrapping_sub(1);
        self.set_flag_bit_if(1, self.registers.y_reg == 0);
        self.set_flag_bit_if(7, self.registers.y_reg.bit(7));
        Ok(start_cycles)
    }

    fn inx(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.x_reg = self.registers.x_reg.wrapping_add(1);
        self.set_flag_bit_if(1, self.registers.x_reg == 0);
        self.set_flag_bit_if(7, self.registers.x_reg.bit(7));
        Ok(start_cycles)
    }

    fn dex(
        &mut self,
        _: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.x_reg = self.registers.x_reg.wrapping_sub(1);
        self.set_flag_bit_if(1, self.registers.x_reg == 0);
        self.set_flag_bit_if(7, self.registers.x_reg.bit(7));
        Ok(start_cycles)
    }

    fn inc(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let new_byte = bus.read_byte(addr)?.wrapping_add(1);
        bus.write_byte(addr, new_byte)?;
        self.set_flag_bit_if(1, new_byte == 0);
        self.set_flag_bit_if(7, new_byte.bit(7));
        Ok(start_cycles)
    }

    fn dec(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let new_byte = bus.read_byte(addr)?.wrapping_sub(1);
        bus.write_byte(addr, new_byte)?;
        self.set_flag_bit_if(1, new_byte == 0);
        self.set_flag_bit_if(7, new_byte.bit(7));
        Ok(start_cycles)
    }

    fn sta(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        bus.write_byte(addr, self.registers.accumulator)?;
        Ok(start_cycles)
    }

    fn stx(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        bus.write_byte(addr as usize, self.registers.x_reg)?;
        Ok(start_cycles)
    }

    fn sty(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        bus.write_byte(addr as usize, self.registers.y_reg)?;
        Ok(start_cycles)
    }

    fn ldy(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let byte = bus.read_byte(addr)?;
        self.registers.y_reg = byte;
        self.set_flag_bit_if(1, byte == 0);
        self.set_flag_bit_if(7, byte.bit(7));
        Ok(start_cycles)
    }

    fn ldx(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let byte = bus.read_byte(addr)?;
        self.registers.x_reg = byte;
        self.set_flag_bit_if(1, byte == 0);
        self.set_flag_bit_if(7, byte.bit(7));
        Ok(start_cycles)
    }

    fn lda(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        let byte = bus.read_byte(addr)?;
        self.registers.accumulator = byte;
        self.set_flag_bit_if(1, byte == 0);
        self.set_flag_bit_if(7, byte.bit(7));
        Ok(start_cycles)
    }

    fn lsr(
        &mut self,
        addr: usize,
        mode: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        match mode {
            AddressMode::ACCUMULATOR => {
                self.set_flag_bit_if(0, self.registers.accumulator.bit(0));
                self.registers.accumulator = self.registers.accumulator >> 1;
                self.set_flag_bit_if(1, self.registers.accumulator == 0);
                self.registers
                    .status_register
                    .modify(Status::NEGATIVE::CLEAR);
            }
            _ => {
                let mut byte = bus.read_byte(addr)?;
                self.set_flag_bit_if(0, byte.bit(0));
                byte = byte >> 1;
                bus.write_byte(addr, byte)?;
                self.set_flag_bit_if(1, byte == 0);
                self.registers
                    .status_register
                    .modify(Status::NEGATIVE::CLEAR);
            }
        }

        Ok(start_cycles)
    }

    fn asl(
        &mut self,
        addr: usize,
        mode: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        match mode {
            AddressMode::ACCUMULATOR => {
                self.set_flag_bit_if(0, self.registers.accumulator.bit(7));
                self.registers.accumulator = self.registers.accumulator << 1;
                self.set_flag_bit_if(1, self.registers.accumulator == 0);
                self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
            }
            _ => {
                let mut byte = bus.read_byte(addr)?;
                self.set_flag_bit_if(0, byte.bit(7));
                byte = byte << 1;
                bus.write_byte(addr, byte)?;
                self.set_flag_bit_if(1, byte == 0);
                self.set_flag_bit_if(7, byte.bit(7));
            }
        }
        Ok(start_cycles)
    }

    fn ror(
        &mut self,
        addr: usize,
        mode: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        match mode {
            AddressMode::ACCUMULATOR => {
                let new_carry = self.registers.accumulator.bit(0);
                self.registers.accumulator = self.registers.accumulator >> 1;
                self.registers
                    .accumulator
                    .set_bit(7, self.registers.status_register.is_set(Status::CARRY));
                self.set_flag_bit_if(0, new_carry);
                self.set_flag_bit_if(1, self.registers.accumulator == 0);
                self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
            }
            _ => {
                let mut byte = bus.read_byte(addr)?;
                let new_carry = byte.bit(0);
                byte = byte >> 1;
                byte.set_bit(7, self.registers.status_register.is_set(Status::CARRY));
                bus.write_byte(addr, byte)?;
                self.set_flag_bit_if(0, new_carry);
                self.set_flag_bit_if(1, byte == 0);
                self.set_flag_bit_if(7, byte.bit(7));
            }
        }
        Ok(start_cycles)
    }

    fn rol(
        &mut self,
        addr: usize,
        mode: AddressMode,
        start_cycles: u8,
        bus: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        match mode {
            AddressMode::ACCUMULATOR => {
                let new_carry = self.registers.accumulator.bit(7);
                self.registers.accumulator = self.registers.accumulator << 1;
                self.registers
                    .accumulator
                    .set_bit(0, self.registers.status_register.is_set(Status::CARRY));
                self.set_flag_bit_if(0, new_carry);
                self.set_flag_bit_if(1, self.registers.accumulator == 0);
                self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
            }
            _ => {
                let mut byte = bus.read_byte(addr)?;
                let new_carry = byte.bit(7);
                byte = byte << 1;
                byte.set_bit(0, self.registers.status_register.is_set(Status::CARRY));
                bus.write_byte(addr, byte)?;
                self.set_flag_bit_if(0, new_carry);
                self.set_flag_bit_if(1, byte == 0);
                self.set_flag_bit_if(7, byte.bit(7));
            }
        }
        Ok(start_cycles)
    }

    fn bcc(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.branchif(addr, false, start_cycles, Status::CARRY)
    }

    fn bcs(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.branchif(addr, true, start_cycles, Status::CARRY)
    }

    fn beq(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.branchif(addr, true, start_cycles, Status::ZERO)
    }

    fn bne(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.branchif(addr, false, start_cycles, Status::ZERO)
    }

    fn bvs(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.branchif(addr, true, start_cycles, Status::OVERFLOW)
    }

    fn bvc(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.branchif(addr, false, start_cycles, Status::OVERFLOW)
    }

    fn bpl(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.branchif(addr, false, start_cycles, Status::NEGATIVE)
    }

    fn bmi(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.branchif(addr, true, start_cycles, Status::NEGATIVE)
    }

    fn branchif(
        &mut self,
        addr: usize,
        set: bool,
        mut cycle_count: u8,
        flag: Field<u8, Register>,
    ) -> Result<u8, &'static str> {
        let check = if set {
            self.registers.status_register.is_set(flag)
        } else {
            !self.registers.status_register.is_set(flag)
        };

        if check {
            cycle_count += 1;
            let new_addr = addr;

            if CPU::will_cross_boundary(new_addr, self.registers.program_counter) {
                cycle_count += 1;
            }

            self.registers.program_counter = new_addr;
        }

        Ok(cycle_count)
    }

    fn jmp(
        &mut self,
        addr: usize,
        _: AddressMode,
        start_cycles: u8,
        _: &mut BusImpl,
    ) -> Result<u8, &'static str> {
        self.registers.program_counter = addr as usize;
        Ok(start_cycles)
    }
}
