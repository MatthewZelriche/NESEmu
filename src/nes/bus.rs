use std::{io::Error, slice::Chunks};

use bitfield::BitRangeMut;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{
    cartridge::Cartridge,
    ppu_registers::{PPURegisters, PPUCTRL, PPUSTATUS},
};

pub struct Bus {
    cartridge: Cartridge,
    cpu_ram: [u8; 2048],
    ppu_ram: [u8; 2048], // TODO: Certain mappers can reroute this memory
    ppu_registers: PPURegisters,
}

impl Bus {
    pub fn new(rom_path: &str) -> Result<Self, Error> {
        Ok(Self {
            cartridge: Cartridge::new(rom_path)?,
            cpu_ram: [0u8; 2048], // Real RAM starts in an uninit state, but rust
            // makes us init it
            ppu_ram: [0u8; 2048],
            ppu_registers: PPURegisters::default(),
        })
    }
}

impl Bus {
    pub fn cpu_read_byte(&mut self, address: usize) -> Result<u8, &'static str> {
        match address {
            (0..=0x1FFF) => Ok(self.cpu_ram[address % 0x0800]),
            (0x2000..=0x3FFF) => self.cpu_read_ppu_register(address, true),
            (0x4000..=0x4017) => Ok(0x0), // TODO: APU
            (0x4020..=0xFFFF) => {
                let prg_addr = self.cartridge.mapper.map_prg_address(address)?;
                Ok(self.cartridge.get_prg_rom()[prg_addr])
            }
            _ => Err("Bad address read on Bus"),
        }
    }

    // Sometimes reading from the CPU bus can cause side effects, eg with PPU registers
    // We need a way to query memory while ensuring this doesn't happen
    pub fn cpu_read_byte_no_modify(&mut self, address: usize) -> Result<u8, &'static str> {
        match address {
            (0x2000..=0x3FFF) => self.cpu_read_ppu_register(address, false),
            _ => Err("Bad address read on Bus"),
        }
    }

    pub fn cpu_read_exact(&mut self, address: usize, buf: &mut [u8]) -> Result<(), &'static str> {
        let len = buf.len();
        for i in 0..len {
            buf[i] = self.cpu_read_byte(address + i)?;
        }
        Ok(())
    }

    pub fn cpu_write_byte(&mut self, address: usize, value: u8) -> Result<(), &'static str> {
        match address {
            (0..=2048) => Ok(self.cpu_ram[address] = value),
            (0x4000..=0x4017) => Ok(()), // TODO: APU
            (0x2000..=0x3FFF) => self.cpu_write_ppu_register(address, value),
            _ => Err("Bad address write on Bus"),
        }
    }

    pub fn cpu_read_ppu_register(
        &mut self,
        address: usize,
        modify: bool,
    ) -> Result<u8, &'static str> {
        match address {
            0x2002 => {
                let val = self.ppu_registers.ppustatus.get();
                if modify {
                    self.ppu_registers
                        .ppustatus
                        .modify(PPUSTATUS::VBLANK::CLEAR); // Clear VBLANK
                    self.ppu_registers.write_latch = false; // Clear write latch
                }
                Ok(val)
            }
            (0x2003..=0x2007) => todo!(),
            _ => Err("Bad Read on PPU register"),
        }
    }

    pub fn cpu_write_ppu_register(
        &mut self,
        address: usize,
        value: u8,
    ) -> Result<(), &'static str> {
        match address {
            0x2000 => Ok(self.ppu_registers.ppuctrl.set(value)),
            0x2001 => Ok(self.ppu_registers.ppustatus.set(value)),
            0x2002 => Ok(self.ppu_registers.ppustatus.set(value)),
            0x2006 => {
                if !self.ppu_registers.write_latch {
                    self.ppu_registers.ppuaddr.set_bit_range(15, 8, value);
                    self.ppu_registers.write_latch = true;
                } else {
                    self.ppu_registers.ppuaddr.set_bit_range(7, 0, value);
                    self.ppu_registers.write_latch = false;
                    // Addresses higher than 0x3FFF get mirrored
                    self.ppu_registers.ppuaddr = self.ppu_registers.ppuaddr % 0x4000;
                }
                Ok(())
            }
            0x2007 => match self.ppu_registers.ppuaddr {
                // TODO: Thigns like CHRRAM and other scenarios where the CPU
                // can write to something thats not a nametable
                (0x2000..=0x2FFF) => {
                    self.ppu_ram[(self.ppu_registers.ppuaddr - 0x2000) as usize] = value;
                    if self.ppu_registers.ppuctrl.is_set(PPUCTRL::VRAM_INC) {
                        // TODO: Does this need a wrapping add?
                        self.ppu_registers.ppuaddr += 32;
                    } else {
                        self.ppu_registers.ppuaddr += 1;
                    }

                    Ok(())
                }
                _ => Err("Bad write to PPU Bus by CPU"),
            },
            (0x2003..=0x2005) => todo!(
                "Attempt to write register: 0x{:X}, ppuaddr: {:X}",
                address,
                self.ppu_registers.ppuaddr
            ),
            _ => Err("Bad Write on PPU Register"),
        }
    }

    pub fn ppu_get_pattern_table(&mut self) -> Chunks<'_, u8> {
        // TODO: handle the two different nametables/mirroring
        self.cartridge.get_chr_rom().chunks(16)
    }

    pub fn ppu_get_registers(&mut self) -> &mut PPURegisters {
        &mut self.ppu_registers
    }
}
