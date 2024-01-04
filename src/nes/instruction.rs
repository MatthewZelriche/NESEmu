use std::io::Write;

use bitfield::{Bit, BitMut};
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

pub enum AddressMode {
    IMPLIED,
    IMMEDIATE,
    ABSOLUTE(bool),
    RELATIVE,
    ZEROPAGE,
    ZEROPAGEX,
    ZEROPAGEY,
    ACCUMULATOR,
    INDIRECTX,
    INDIRECTY,
    ABSOLUTEX,
    ABSOLUTEY,
    INDIRECT,
}

pub struct Opcode {
    mnemonic: &'static str,
    mode: AddressMode,
    bytes: [u8; 3],
    num_bytes: u8,
    cycles: u8,
    execute: for<'a> fn(&'a mut CPU, usize, &'a Opcode, &'a mut Bus) -> Result<u8, &'static str>,
}

impl CPU {
    pub fn lookup_opcode(&mut self, opcode: u8, bus: &mut Bus) -> Result<Opcode, &'static str> {
        match opcode {
            0x00 => todo!(),
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
            0x11 => Ok(Opcode {
                mnemonic: "ORA",
                mode: AddressMode::INDIRECTY,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ora,
            }),
            0x15 => Ok(Opcode {
                mnemonic: "ORA",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ora,
            }),
            0x16 => Ok(Opcode {
                mnemonic: "ASL",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::asl,
            }),
            0x19 => Ok(Opcode {
                mnemonic: "ORA",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ora,
            }),
            0x1D => Ok(Opcode {
                mnemonic: "ORA",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ora,
            }),
            0x1E => Ok(Opcode {
                mnemonic: "ASL",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 7,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::asl,
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
            0x31 => Ok(Opcode {
                mnemonic: "AND",
                mode: AddressMode::INDIRECTY,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::and,
            }),
            0x35 => Ok(Opcode {
                mnemonic: "AND",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::and,
            }),
            0x36 => Ok(Opcode {
                mnemonic: "ROL",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::rol,
            }),
            0x38 => Ok(Opcode {
                mnemonic: "SEC",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::sec,
            }),
            0x39 => Ok(Opcode {
                mnemonic: "AND",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::and,
            }),
            0x3D => Ok(Opcode {
                mnemonic: "AND",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::and,
            }),
            0x3E => Ok(Opcode {
                mnemonic: "ROL",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 7,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::rol,
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
            0x51 => Ok(Opcode {
                mnemonic: "EOR",
                mode: AddressMode::INDIRECTY,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::eor,
            }),
            0x55 => Ok(Opcode {
                mnemonic: "EOR",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::eor,
            }),
            0x56 => Ok(Opcode {
                mnemonic: "LSR",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::lsr,
            }),
            0x59 => Ok(Opcode {
                mnemonic: "EOR",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::eor,
            }),
            0x5D => Ok(Opcode {
                mnemonic: "EOR",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::eor,
            }),
            0x5E => Ok(Opcode {
                mnemonic: "LSR",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 7,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::lsr,
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
            0x6C => Ok(Opcode {
                mnemonic: "JMP",
                mode: AddressMode::INDIRECT,
                num_bytes: 3,
                cycles: 5,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::jmp,
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
            0x71 => Ok(Opcode {
                mnemonic: "ADC",
                mode: AddressMode::INDIRECTY,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::adc,
            }),
            0x75 => Ok(Opcode {
                mnemonic: "ADC",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::adc,
            }),
            0x76 => Ok(Opcode {
                mnemonic: "ROR",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ror,
            }),
            0x78 => Ok(Opcode {
                mnemonic: "SEI",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::sei,
            }),
            0x79 => Ok(Opcode {
                mnemonic: "ADC",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::adc,
            }),
            0x7D => Ok(Opcode {
                mnemonic: "ADC",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::adc,
            }),
            0x7E => Ok(Opcode {
                mnemonic: "ROR",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 7,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ror,
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
            0x91 => Ok(Opcode {
                mnemonic: "STA",
                mode: AddressMode::INDIRECTY,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sta,
            }),
            0x94 => Ok(Opcode {
                mnemonic: "STY",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sty,
            }),
            0x95 => Ok(Opcode {
                mnemonic: "STA",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sta,
            }),
            0x96 => Ok(Opcode {
                mnemonic: "STX",
                mode: AddressMode::ZEROPAGEY,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::stx,
            }),
            0x98 => Ok(Opcode {
                mnemonic: "TYA",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::tya,
            }),
            0x99 => Ok(Opcode {
                mnemonic: "STA",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 5,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::sta,
            }),
            0x9A => Ok(Opcode {
                mnemonic: "TXS",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::txs,
            }),
            0x9D => Ok(Opcode {
                mnemonic: "STA",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 5,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::sta,
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
            0xB4 => Ok(Opcode {
                mnemonic: "LDY",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ldy,
            }),
            0xB5 => Ok(Opcode {
                mnemonic: "LDA",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::lda,
            }),
            0xB6 => Ok(Opcode {
                mnemonic: "LDX",
                mode: AddressMode::ZEROPAGEY,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::ldx,
            }),
            0xB8 => Ok(Opcode {
                mnemonic: "CLV",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::clv,
            }),
            0xB9 => Ok(Opcode {
                mnemonic: "LDA",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::lda,
            }),
            0xBA => Ok(Opcode {
                mnemonic: "TSX",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::tsx,
            }),
            0xBC => Ok(Opcode {
                mnemonic: "LDY",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ldy,
            }),
            0xBD => Ok(Opcode {
                mnemonic: "LDA",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::lda,
            }),
            0xBE => Ok(Opcode {
                mnemonic: "LDX",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::ldx,
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
            0xD1 => Ok(Opcode {
                mnemonic: "CMP",
                mode: AddressMode::INDIRECTY,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cmp,
            }),
            0xD5 => Ok(Opcode {
                mnemonic: "CMP",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::cmp,
            }),
            0xD6 => Ok(Opcode {
                mnemonic: "DEC",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::dec,
            }),
            0xD8 => Ok(Opcode {
                mnemonic: "CLD",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::cld,
            }),
            0xD9 => Ok(Opcode {
                mnemonic: "CMP",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::cmp,
            }),
            0xDD => Ok(Opcode {
                mnemonic: "CMP",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::cmp,
            }),
            0xDE => Ok(Opcode {
                mnemonic: "DEC",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 7,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::dec,
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
            0xF1 => Ok(Opcode {
                mnemonic: "SBC",
                mode: AddressMode::INDIRECTY,
                num_bytes: 2,
                cycles: 5,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sbc,
            }),
            0xF5 => Ok(Opcode {
                mnemonic: "SBC",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 4,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::sbc,
            }),
            0xF6 => Ok(Opcode {
                mnemonic: "INC",
                mode: AddressMode::ZEROPAGEX,
                num_bytes: 2,
                cycles: 6,
                bytes: self.fetch_one_more_bytes(opcode, bus)?,
                execute: CPU::inc,
            }),
            0xF8 => Ok(Opcode {
                mnemonic: "SED",
                mode: AddressMode::IMPLIED,
                num_bytes: 1,
                cycles: 2,
                bytes: self.fetch_zero_more_bytes(opcode),
                execute: CPU::sed,
            }),
            0xF9 => Ok(Opcode {
                mnemonic: "SBC",
                mode: AddressMode::ABSOLUTEY,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::sbc,
            }),
            0xFD => Ok(Opcode {
                mnemonic: "SBC",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 4,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::sbc,
            }),
            0xFE => Ok(Opcode {
                mnemonic: "INC",
                mode: AddressMode::ABSOLUTEX,
                num_bytes: 3,
                cycles: 7,
                bytes: self.fetch_two_more_bytes(opcode, bus)?,
                execute: CPU::inc,
            }),
            _ => Err("Invalid opcode"),
        }
    }

    pub fn execute_opcode<'a>(
        &'a mut self,
        opcode_val: u8,
        bus: &'a mut Bus,
    ) -> Result<u8, &'static str> {
        let opcode = self.lookup_opcode(opcode_val, bus)?;
        // We don't care if this succeeds or not, since the logging info is optional
        //let _ = self.write_opcode(&opcode, bus);

        let addr = self.fetch_operand_address(&opcode, bus)?;
        (opcode.execute)(self, addr, &opcode, bus)
    }

    fn fetch_zero_more_bytes(&mut self, opcode: u8) -> [u8; 3] {
        [opcode, 0x0, 0x0]
    }

    fn fetch_one_more_bytes(&mut self, opcode: u8, bus: &mut Bus) -> Result<[u8; 3], &'static str> {
        let bytes = [
            opcode,
            bus.cpu_read_byte(self.registers.program_counter)?,
            0x0,
        ];
        self.registers.program_counter += 1;
        Ok(bytes)
    }

    fn fetch_two_more_bytes(&mut self, opcode: u8, bus: &mut Bus) -> Result<[u8; 3], &'static str> {
        let mut bytes = [opcode, 0x0, 0x0];
        bus.cpu_read_exact(self.registers.program_counter, &mut bytes[1..])?;
        self.registers.program_counter += 2;
        Ok(bytes)
    }

    fn fetch_operand_address(
        &mut self,
        opcode: &Opcode,
        bus: &mut Bus,
    ) -> Result<usize, &'static str> {
        match opcode.mode {
            AddressMode::IMPLIED | AddressMode::ACCUMULATOR => Ok(0x0), // Address is irrelevant for implied and ACC
            AddressMode::IMMEDIATE => Ok(self.registers.program_counter - 1),
            AddressMode::ABSOLUTE(_) => {
                Ok(u16::from_le_bytes(opcode.bytes[1..].try_into().unwrap()) as usize)
            }
            AddressMode::RELATIVE => {
                // Relative uses a SIGNED offset!
                // TODO: This may crash if relative is allowed to address in a wrapping fashion (eg 0xFFFF -> zero page)
                let signed_operand = i8::from_le_bytes([opcode.bytes[1]]) as isize;
                Ok((signed_operand + self.registers.program_counter as isize) as usize)
            }
            AddressMode::ZEROPAGE => Ok(opcode.bytes[1] as usize),
            AddressMode::INDIRECTX => {
                // Indirect zeropage is tricky, because if we are given a lsb of 0xFF,
                // we need to discover the high byte at 0x00, not 0x100
                let lsb_addr = opcode.bytes[1].wrapping_add(self.registers.x_reg) as usize;
                let msb_addr = (lsb_addr as u8).wrapping_add(1) as usize;
                let addr_bytes = [bus.cpu_read_byte(lsb_addr)?, bus.cpu_read_byte(msb_addr)?];
                Ok(u16::from_le_bytes(addr_bytes) as usize)
            }
            AddressMode::INDIRECTY => {
                let addr = self.fetch_indirect_y_base_addr(opcode, bus)? as u16;
                Ok(addr.wrapping_add(self.registers.y_reg as u16) as usize)
            }
            AddressMode::ABSOLUTEX => Ok((self.fetch_absolute_base_addr(opcode) as u16)
                .wrapping_add(self.registers.x_reg as u16)
                as usize),
            AddressMode::ABSOLUTEY => Ok((self.fetch_absolute_base_addr(opcode) as u16)
                .wrapping_add(self.registers.y_reg as u16)
                as usize),
            AddressMode::INDIRECT => {
                let base_addr = self.fetch_absolute_base_addr(opcode);
                // Have to emulate a cpu bug with indirect mode
                let base_addr_msb_wrap = CPU::PAGE_SZ_MASK & base_addr;
                let base_addr_msb =
                    (((base_addr + 1) % base_addr_msb_wrap) as u8) as usize | base_addr_msb_wrap;
                let addr_bytes = [
                    bus.cpu_read_byte(base_addr)?,
                    bus.cpu_read_byte(base_addr_msb)?,
                ];
                Ok(u16::from_le_bytes(addr_bytes) as usize)
            }
            AddressMode::ZEROPAGEX => {
                let base_addr = opcode.bytes[1];
                Ok(base_addr.wrapping_add(self.registers.x_reg) as usize)
            }
            AddressMode::ZEROPAGEY => {
                let base_addr = opcode.bytes[1];
                Ok(base_addr.wrapping_add(self.registers.y_reg) as usize)
            }
        }
    }

    pub fn fetch_indirect_y_base_addr(
        &self,
        opcode: &Opcode,
        bus: &mut Bus,
    ) -> Result<usize, &'static str> {
        let lsb_addr = opcode.bytes[1] as usize;
        let msb_addr = (lsb_addr as u8).wrapping_add(1) as usize;
        let addr_bytes = [bus.cpu_read_byte(lsb_addr)?, bus.cpu_read_byte(msb_addr)?];
        Ok(u16::from_le_bytes(addr_bytes) as usize)
    }

    pub fn fetch_absolute_base_addr(&self, opcode: &Opcode) -> usize {
        u16::from_le_bytes([opcode.bytes[1], opcode.bytes[2]]) as usize
    }

    pub fn adjust_cycles(
        &mut self,
        addr: usize,
        opcode: &Opcode,
        bus: &mut Bus,
    ) -> Result<u8, &'static str> {
        let mut cycles = opcode.cycles;
        let base_addr = match opcode.mode {
            AddressMode::INDIRECTY => self.fetch_indirect_y_base_addr(opcode, bus)?,
            AddressMode::ABSOLUTEX => self.fetch_absolute_base_addr(opcode),
            AddressMode::ABSOLUTEY => self.fetch_absolute_base_addr(opcode),
            _ => return Ok(cycles),
        };

        if CPU::will_cross_boundary(base_addr, addr) {
            cycles += 1;
        }
        Ok(cycles)
    }

    fn rti(&mut self, _: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let mut byte = [0];
        self.pop_stack(&mut byte, bus)?;
        self.set_status_register(byte[0]);

        let mut pc = [0u8; 2];
        self.pop_stack(&mut pc, bus)?;
        self.registers.program_counter = u16::from_le_bytes(pc) as usize;

        Ok(opcode.cycles)
    }

    fn sbc(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let old_accumulator = self.registers.accumulator;
        let mut mem = bus.cpu_read_byte(addr)?;
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

        self.adjust_cycles(addr, opcode, bus)
    }

    fn adc(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let old_accumulator = self.registers.accumulator;
        let mem = bus.cpu_read_byte(addr)?;
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

        self.adjust_cycles(addr, opcode, bus)
    }

    fn plp(&mut self, _: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let mut byte = [0u8];
        self.pop_stack(&mut byte, bus)?;
        self.set_status_register(byte[0]);
        Ok(opcode.cycles)
    }

    fn pla(&mut self, _: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let mut byte = [0u8];
        self.pop_stack(&mut byte, bus)?;
        self.registers.accumulator = byte[0];
        self.set_flag_bit_if(1, byte[0] == 0);
        self.set_flag_bit_if(7, byte[0].bit(7));
        Ok(opcode.cycles)
    }

    fn php(&mut self, _: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        // Instructions that push status flags to the stack always push BFLAG as set
        let mut copy = self.registers.status_register.extract();
        copy.modify(Status::BFLAG::SET);

        let byte = [copy.get()];
        self.push_stack(&byte, bus)?;
        Ok(opcode.cycles)
    }

    fn pha(&mut self, _: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let byte = [self.registers.accumulator];
        self.push_stack(&byte, bus)?;
        Ok(opcode.cycles)
    }

    fn nop(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        Ok(opcode.cycles)
    }

    fn clc(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.status_register.modify(Status::CARRY::CLEAR);
        Ok(opcode.cycles)
    }

    fn sei(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers
            .status_register
            .modify(Status::INT_DISABLE::SET);
        Ok(opcode.cycles)
    }

    fn sed(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.status_register.modify(Status::DECIMAL::SET);
        Ok(opcode.cycles)
    }

    fn clv(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers
            .status_register
            .modify(Status::OVERFLOW::CLEAR);
        Ok(opcode.cycles)
    }

    fn cld(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers
            .status_register
            .modify(Status::DECIMAL::CLEAR);
        Ok(opcode.cycles)
    }

    fn sec(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.status_register.modify(Status::CARRY::SET);
        Ok(opcode.cycles)
    }

    fn and(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        self.registers.accumulator &= bus.cpu_read_byte(addr)?;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        self.adjust_cycles(addr, opcode, bus)
    }

    fn ora(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        self.registers.accumulator |= bus.cpu_read_byte(addr)?;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        self.adjust_cycles(addr, opcode, bus)
    }

    fn eor(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        self.registers.accumulator ^= bus.cpu_read_byte(addr)?;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        self.adjust_cycles(addr, opcode, bus)
    }

    fn jsr(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        // Store the current program counter (which, right now, points to the NEXT
        // instruction after the one we are processing)
        // big endian because we need to push to the stack in reverse order of how they should be
        self.push_stack(
            &u16::to_be_bytes((self.registers.program_counter - 1) as u16),
            bus,
        )?;
        self.registers.program_counter = addr;
        Ok(opcode.cycles)
    }

    fn rts(&mut self, _: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let mut addr_bytes = [0u8; 2];
        self.pop_stack(&mut addr_bytes, bus)?;
        self.registers.program_counter = (u16::from_le_bytes(addr_bytes) + 1) as usize;
        Ok(opcode.cycles)
    }

    fn bit(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let byte = bus.cpu_read_byte(addr)?;
        self.set_flag_bit_if(1, self.registers.accumulator & byte == 0);
        self.set_flag_bit_if(6, byte.bit(6));
        self.set_flag_bit_if(7, byte.bit(7));

        Ok(opcode.cycles)
    }

    fn cmp(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        self.compare_reg(addr, self.registers.accumulator, opcode, bus)
    }

    fn cpy(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        self.compare_reg(addr, self.registers.y_reg, opcode, bus)
    }

    fn cpx(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        self.compare_reg(addr, self.registers.x_reg, opcode, bus)
    }

    fn compare_reg(
        &mut self,
        addr: usize,
        reg_val: u8,
        opcode: &Opcode,
        bus: &mut Bus,
    ) -> Result<u8, &'static str> {
        let byte = bus.cpu_read_byte(addr)?;
        self.set_flag_bit_if(0, reg_val >= byte);
        self.set_flag_bit_if(1, reg_val == byte);
        self.set_flag_bit_if(7, reg_val.wrapping_sub(byte).bit(7));
        self.adjust_cycles(addr, opcode, bus)
    }

    fn tay(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.y_reg = self.registers.accumulator;
        self.set_flag_bit_if(1, self.registers.y_reg == 0);
        self.set_flag_bit_if(7, self.registers.y_reg.bit(7));
        Ok(opcode.cycles)
    }

    fn tya(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.accumulator = self.registers.y_reg;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        Ok(opcode.cycles)
    }

    fn tax(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.x_reg = self.registers.accumulator;
        self.set_flag_bit_if(1, self.registers.x_reg == 0);
        self.set_flag_bit_if(7, self.registers.x_reg.bit(7));
        Ok(opcode.cycles)
    }

    fn txa(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.accumulator = self.registers.x_reg;
        self.set_flag_bit_if(1, self.registers.accumulator == 0);
        self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
        Ok(opcode.cycles)
    }

    fn tsx(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.x_reg = self.registers.stack_ptr as u8;
        self.set_flag_bit_if(1, self.registers.x_reg == 0);
        self.set_flag_bit_if(7, self.registers.x_reg.bit(7));
        Ok(opcode.cycles)
    }

    fn txs(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.stack_ptr = self.registers.x_reg as usize;
        Ok(opcode.cycles)
    }

    fn iny(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.y_reg = self.registers.y_reg.wrapping_add(1);
        self.set_flag_bit_if(1, self.registers.y_reg == 0);
        self.set_flag_bit_if(7, self.registers.y_reg.bit(7));
        Ok(opcode.cycles)
    }

    fn dey(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.y_reg = self.registers.y_reg.wrapping_sub(1);
        self.set_flag_bit_if(1, self.registers.y_reg == 0);
        self.set_flag_bit_if(7, self.registers.y_reg.bit(7));
        Ok(opcode.cycles)
    }

    fn inx(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.x_reg = self.registers.x_reg.wrapping_add(1);
        self.set_flag_bit_if(1, self.registers.x_reg == 0);
        self.set_flag_bit_if(7, self.registers.x_reg.bit(7));
        Ok(opcode.cycles)
    }

    fn dex(&mut self, _: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.x_reg = self.registers.x_reg.wrapping_sub(1);
        self.set_flag_bit_if(1, self.registers.x_reg == 0);
        self.set_flag_bit_if(7, self.registers.x_reg.bit(7));
        Ok(opcode.cycles)
    }

    fn inc(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let new_byte = bus.cpu_read_byte(addr)?.wrapping_add(1);
        bus.cpu_write_byte(addr, new_byte)?;
        self.set_flag_bit_if(1, new_byte == 0);
        self.set_flag_bit_if(7, new_byte.bit(7));
        Ok(opcode.cycles)
    }

    fn dec(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let new_byte = bus.cpu_read_byte(addr)?.wrapping_sub(1);
        bus.cpu_write_byte(addr, new_byte)?;
        self.set_flag_bit_if(1, new_byte == 0);
        self.set_flag_bit_if(7, new_byte.bit(7));
        Ok(opcode.cycles)
    }

    fn sta(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        bus.cpu_write_byte(addr, self.registers.accumulator)?;
        Ok(opcode.cycles)
    }

    fn stx(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        bus.cpu_write_byte(addr as usize, self.registers.x_reg)?;
        Ok(opcode.cycles)
    }

    fn sty(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        bus.cpu_write_byte(addr as usize, self.registers.y_reg)?;
        Ok(opcode.cycles)
    }

    fn ldy(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let byte = bus.cpu_read_byte(addr)?;
        self.registers.y_reg = byte;
        self.set_flag_bit_if(1, byte == 0);
        self.set_flag_bit_if(7, byte.bit(7));
        self.adjust_cycles(addr, opcode, bus)
    }

    fn ldx(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let byte = bus.cpu_read_byte(addr)?;
        self.registers.x_reg = byte;
        self.set_flag_bit_if(1, byte == 0);
        self.set_flag_bit_if(7, byte.bit(7));
        self.adjust_cycles(addr, opcode, bus)
    }

    fn lda(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        let byte = bus.cpu_read_byte(addr)?;
        self.registers.accumulator = byte;
        self.set_flag_bit_if(1, byte == 0);
        self.set_flag_bit_if(7, byte.bit(7));
        self.adjust_cycles(addr, opcode, bus)
    }

    fn lsr(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        match opcode.mode {
            AddressMode::ACCUMULATOR => {
                self.set_flag_bit_if(0, self.registers.accumulator.bit(0));
                self.registers.accumulator = self.registers.accumulator >> 1;
                self.set_flag_bit_if(1, self.registers.accumulator == 0);
                self.registers
                    .status_register
                    .modify(Status::NEGATIVE::CLEAR);
            }
            _ => {
                let mut byte = bus.cpu_read_byte(addr)?;
                self.set_flag_bit_if(0, byte.bit(0));
                byte = byte >> 1;
                bus.cpu_write_byte(addr, byte)?;
                self.set_flag_bit_if(1, byte == 0);
                self.registers
                    .status_register
                    .modify(Status::NEGATIVE::CLEAR);
            }
        }

        Ok(opcode.cycles)
    }

    fn asl(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        match opcode.mode {
            AddressMode::ACCUMULATOR => {
                self.set_flag_bit_if(0, self.registers.accumulator.bit(7));
                self.registers.accumulator = self.registers.accumulator << 1;
                self.set_flag_bit_if(1, self.registers.accumulator == 0);
                self.set_flag_bit_if(7, self.registers.accumulator.bit(7));
            }
            _ => {
                let mut byte = bus.cpu_read_byte(addr)?;
                self.set_flag_bit_if(0, byte.bit(7));
                byte = byte << 1;
                bus.cpu_write_byte(addr, byte)?;
                self.set_flag_bit_if(1, byte == 0);
                self.set_flag_bit_if(7, byte.bit(7));
            }
        }
        Ok(opcode.cycles)
    }

    fn ror(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        match opcode.mode {
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
                let mut byte = bus.cpu_read_byte(addr)?;
                let new_carry = byte.bit(0);
                byte = byte >> 1;
                byte.set_bit(7, self.registers.status_register.is_set(Status::CARRY));
                bus.cpu_write_byte(addr, byte)?;
                self.set_flag_bit_if(0, new_carry);
                self.set_flag_bit_if(1, byte == 0);
                self.set_flag_bit_if(7, byte.bit(7));
            }
        }
        Ok(opcode.cycles)
    }

    fn rol(&mut self, addr: usize, opcode: &Opcode, bus: &mut Bus) -> Result<u8, &'static str> {
        match opcode.mode {
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
                let mut byte = bus.cpu_read_byte(addr)?;
                let new_carry = byte.bit(7);
                byte = byte << 1;
                byte.set_bit(0, self.registers.status_register.is_set(Status::CARRY));
                bus.cpu_write_byte(addr, byte)?;
                self.set_flag_bit_if(0, new_carry);
                self.set_flag_bit_if(1, byte == 0);
                self.set_flag_bit_if(7, byte.bit(7));
            }
        }
        Ok(opcode.cycles)
    }

    fn bcc(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.branchif(addr, false, opcode.cycles, Status::CARRY)
    }

    fn bcs(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.branchif(addr, true, opcode.cycles, Status::CARRY)
    }

    fn beq(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.branchif(addr, true, opcode.cycles, Status::ZERO)
    }

    fn bne(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.branchif(addr, false, opcode.cycles, Status::ZERO)
    }

    fn bvs(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.branchif(addr, true, opcode.cycles, Status::OVERFLOW)
    }

    fn bvc(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.branchif(addr, false, opcode.cycles, Status::OVERFLOW)
    }

    fn bpl(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.branchif(addr, false, opcode.cycles, Status::NEGATIVE)
    }

    fn bmi(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.branchif(addr, true, opcode.cycles, Status::NEGATIVE)
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

    fn jmp(&mut self, addr: usize, opcode: &Opcode, _: &mut Bus) -> Result<u8, &'static str> {
        self.registers.program_counter = addr as usize;
        Ok(opcode.cycles)
    }

    // TODO: This breaks everything because it performs modifying reads
    pub fn write_opcode(&mut self, opcode: &Opcode, bus: &mut Bus) -> Result<(), &'static str> {
        let mut fmt_string = format!("{:04X}  ", self.current_instruction_addr);

        if opcode.num_bytes == 1 {
            fmt_string = format!(
                "{}{:02X}{:<8}{} ",
                fmt_string, opcode.bytes[0], "", opcode.mnemonic
            );

            match opcode.mode {
                AddressMode::ACCUMULATOR => {
                    fmt_string = format!("{}A ", fmt_string);
                }
                _ => {}
            }
        } else if opcode.num_bytes == 2 {
            fmt_string = format!(
                "{}{:02X} {:02X}{:<5}{} ",
                fmt_string, opcode.bytes[0], opcode.bytes[1], "", opcode.mnemonic
            );

            match opcode.mode {
                AddressMode::IMMEDIATE => {
                    fmt_string = format!("{}#${:02X}", fmt_string, opcode.bytes[1]);
                }
                AddressMode::RELATIVE => {
                    fmt_string = format!(
                        "{}${:02X}",
                        fmt_string,
                        self.fetch_operand_address(opcode, bus)?
                    );
                }
                AddressMode::ZEROPAGE => {
                    let address_value = bus.cpu_read_byte(opcode.bytes[1] as usize)?;
                    fmt_string = format!(
                        "{}${:02X} = {:02X}",
                        fmt_string, opcode.bytes[1], address_value
                    );
                }
                AddressMode::INDIRECTX => {
                    let lsb_addr = opcode.bytes[1].wrapping_add(self.registers.x_reg);
                    let addr = self.fetch_operand_address(opcode, bus)?;
                    fmt_string = format!(
                        "{}(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                        fmt_string,
                        opcode.bytes[1],
                        lsb_addr,
                        addr,
                        bus.cpu_read_byte(addr)?
                    );
                }
                AddressMode::INDIRECTY => {
                    let addr = self.fetch_operand_address(opcode, bus)?;
                    fmt_string = format!(
                        "{}(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                        fmt_string,
                        opcode.bytes[1],
                        self.fetch_indirect_y_base_addr(opcode, bus)?,
                        addr,
                        bus.cpu_read_byte(addr)?
                    );
                }
                AddressMode::ZEROPAGEX => {
                    let addr = self.fetch_operand_address(opcode, bus)?;
                    fmt_string = format!(
                        "{}${:02X},X @ {:02X} = {:02X}",
                        fmt_string,
                        opcode.bytes[1],
                        addr,
                        bus.cpu_read_byte(addr)?
                    );
                }
                AddressMode::ZEROPAGEY => {
                    let addr = self.fetch_operand_address(opcode, bus)?;
                    fmt_string = format!(
                        "{}${:02X},Y @ {:02X} = {:02X}",
                        fmt_string,
                        opcode.bytes[1],
                        addr,
                        bus.cpu_read_byte(addr)?
                    );
                }
                _ => {} // should never happen
            }
        } else if opcode.num_bytes == 3 {
            fmt_string = format!(
                "{}{:02X} {:02X} {:02X}  {} ",
                fmt_string, opcode.bytes[0], opcode.bytes[1], opcode.bytes[2], opcode.mnemonic
            );

            match opcode.mode {
                AddressMode::ABSOLUTE(mem_modify) => {
                    let operand = u16::from_le_bytes(opcode.bytes[1..].try_into().unwrap());
                    fmt_string = format!("{}${:04X}", fmt_string, operand);

                    if mem_modify {
                        let byte = bus.cpu_read_byte(operand as usize)?;
                        fmt_string = format!("{} = {:02X}", fmt_string, byte);
                    }
                }
                AddressMode::INDIRECT => {
                    let addr = self.fetch_operand_address(opcode, bus)?;
                    let base_addr = self.fetch_absolute_base_addr(opcode);
                    fmt_string = format!("{}(${:04X}) = {:04X}", fmt_string, base_addr, addr);
                }
                AddressMode::ABSOLUTEY => {
                    let addr = self.fetch_operand_address(opcode, bus)?;
                    let base_addr = self.fetch_absolute_base_addr(opcode);
                    fmt_string = format!(
                        "{}${:04X},Y @ {:04X} = {:02X}",
                        fmt_string,
                        base_addr,
                        addr,
                        bus.cpu_read_byte(addr)?
                    );
                }
                AddressMode::ABSOLUTEX => {
                    let addr = self.fetch_operand_address(opcode, bus)?;
                    let base_addr = self.fetch_absolute_base_addr(opcode);
                    fmt_string = format!(
                        "{}${:04X},X @ {:04X} = {:02X}",
                        fmt_string,
                        base_addr,
                        addr,
                        bus.cpu_read_byte(addr)?
                    );
                }

                _ => {} // should never happen
            }
        }

        fmt_string = format!("{:<42}", fmt_string);
        fmt_string = format!(
            "{}     {} CYC:{}",
            fmt_string, self.old_register_state, self.total_cycles
        );
        write!(self.log_file, "{}\n", fmt_string).map_err(|_| "Failed to write to log file")?;
        log::info!("{}", fmt_string);
        Ok(())
    }
}
