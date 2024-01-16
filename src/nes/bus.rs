use std::{io::Error, slice::Chunks};

use bitfield::{Bit, BitRangeMut};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{
    cartridge::Cartridge,
    controller::Controller,
    ines::{Flags1, INESHeader},
    palette_memory::PaletteMemory,
    ppu_registers::{PPURegisters, PPUCTRL, PPUSTATUS},
};

pub struct Bus {
    cartridge: Cartridge,
    cpu_ram: [u8; 2048],
    ppu_ram: [u8; 2048], // TODO: Certain mappers can reroute this memory
    pub oam_ram: [u8; 256],
    oam_addr: u8,
    pending_dma: bool,
    dma_page_addr: usize,
    ppu_registers: PPURegisters,
    pub palette_memory: PaletteMemory,
    pub controller: Controller,
}

impl Bus {
    pub fn new(rom_path: &str) -> Result<Self, Error> {
        Ok(Self {
            cartridge: Cartridge::new(rom_path)?,
            cpu_ram: [0u8; 2048], // Real RAM starts in an uninit state, but rust
            // makes us init it
            ppu_ram: [0u8; 2048],
            oam_ram: [0u8; 256],
            oam_addr: 0,
            pending_dma: false,
            dma_page_addr: 0,
            ppu_registers: PPURegisters::default(),
            palette_memory: PaletteMemory::new(),
            controller: Controller::new(),
        })
    }
}

impl Bus {
    pub fn pending_dma(&self) -> bool {
        self.pending_dma
    }

    pub fn process_dma(&mut self) {
        for addr in self.dma_page_addr..self.dma_page_addr + 0x100 {
            self.cpu_write_ppu_register(0x2004, self.cpu_ram[addr])
                .unwrap();
        }

        self.pending_dma = false;
    }

    pub fn cartridge_header(&self) -> &INESHeader {
        &self.cartridge.get_header()
    }

    pub fn cpu_read_byte(&mut self, address: usize) -> Result<u8, &'static str> {
        match address {
            (0..=0x1FFF) => Ok(self.cpu_ram[address % 0x0800]),
            (0x2000..=0x3FFF) => self.cpu_read_ppu_register(address, true),
            (0x4000..=0x4015) => Ok(0x0), // TODO: APU
            0x4016 => Ok(self.controller.read_from_controller()),
            0x4017 => Ok(0x0), // Currently not supported
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
            (0..=0x1FFF) => Ok(self.cpu_ram[address % 0x0800]),
            (0x2000..=0x3FFF) => self.cpu_read_ppu_register(address, false),
            (0x4000..=0x4017) => Ok(0x0), // TODO: APU
            // TODO: Controller
            (0x4020..=0xFFFF) => {
                let prg_addr = self.cartridge.mapper.map_prg_address(address)?;
                Ok(self.cartridge.get_prg_rom()[prg_addr])
            }
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
            (0x4000..=0x4013) => Ok(()), // TODO: APU
            0x4014 => {
                self.dma_page_addr = (value as usize) << 8;
                Ok(self.pending_dma = true)
            }
            0x4015 => Ok(()), // TODO: APU?
            0x4016 => Ok(self.controller.write_to_controller(value.bit(0))),
            0x4017 => Ok(()), // Currently not supported
            (0x2000..=0x3FFF) => self.cpu_write_ppu_register(address, value),
            (0x4020..=0xFFFF) => self.cartridge.mapper.write_register(address, value),
            _ => Err("Bad address write on Bus"),
        }
    }

    pub fn cpu_read_ppu_register(
        &mut self,
        address: usize,
        modify: bool,
    ) -> Result<u8, &'static str> {
        match address {
            0x2000 => Ok(self.ppu_registers.ppuctrl.get()),
            0x2001 => Ok(self.ppu_registers.ppumask.get()),
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
            0x2003 => Ok(self.oam_addr),
            0x2004 => Ok(self.oam_ram[self.oam_addr as usize]),
            0x2005 => Ok(0x0), // TODO
            0x2006 => todo!(), // How to handle this? It's two bytes
            0x2007 => {
                let final_res = match self.ppu_registers.ppuaddr {
                    (0..=0x1FFF) => {
                        // Buffered read
                        let res = self.ppu_registers.ppudata;
                        // Then fetch new data
                        self.ppu_registers.ppudata =
                            self.cartridge.get_chr_rom()[self.ppu_registers.ppuaddr as usize];
                        Ok(res)
                    }
                    (0x2000..=0x2FFF) => {
                        // Buffered read
                        let res = self.ppu_registers.ppudata;
                        // Then fetch new data
                        self.ppu_registers.ppudata =
                            self.ppu_ram[self.translate_nametable_addr(self.ppu_registers.ppuaddr)];
                        Ok(res)
                    }
                    (0x3F00..=0x3FFF) => {
                        // No buffered read
                        Ok(self
                            .palette_memory
                            .get_entry(0x3F00 | (self.ppu_registers.ppuaddr as usize % 0x20)))
                    }
                    _ => return Err("Bad read from PPU Bus by CPU"),
                };

                self.ppu_increment_vram_ptr();
                final_res
            }
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
            0x2001 => Ok(self.ppu_registers.ppumask.set(value)),
            0x2002 => Ok(self.ppu_registers.ppustatus.set(value)),
            0x2003 => Ok(self.oam_addr = value), // TODO: Needs to be set to 0 during vblank (?)
            0x2004 => {
                self.oam_ram[self.oam_addr as usize] = value;
                Ok(self.oam_addr = self.oam_addr.wrapping_add(1))
            }
            0x2005 => {
                if !self.ppu_registers.write_latch {
                    self.ppu_registers.fine_x = value;
                    self.ppu_registers.write_latch = true;
                } else {
                    self.ppu_registers.fine_y = value;
                    self.ppu_registers.write_latch = false;
                }
                Ok(())
            }
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
            0x2007 => {
                match self.ppu_registers.ppuaddr {
                    (0x0000..=0x1FFF) => {
                        // Attempt to write to CHR RAM...
                        if let Some(ram) = self.cartridge.get_chr_ram() {
                            ram[self.ppu_registers.ppuaddr as usize] = value;
                        } else {
                            // If there is no CHR RAM we treat it as a no op...
                            return Ok(());
                        }
                    }
                    (0x2000..=0x2FFF) => {
                        self.ppu_ram[self.translate_nametable_addr(self.ppu_registers.ppuaddr)] =
                            value;
                    }
                    (0x3F00..=0x3FFF) => {
                        self.palette_memory.set_entry(
                            0x3F00 | (self.ppu_registers.ppuaddr as usize % 0x20),
                            value,
                        );
                    }
                    _ => return Err("Bad write to PPU Bus by CPU"),
                };

                self.ppu_increment_vram_ptr();

                Ok(())
            }
            _ => Err("Bad Write on PPU Register"),
        }
    }

    fn ppu_increment_vram_ptr(&mut self) {
        if self.ppu_registers.ppuctrl.is_set(PPUCTRL::VRAM_INC) {
            // TODO: Does this need a wrapping add?
            self.ppu_registers.ppuaddr += 32;
        } else {
            self.ppu_registers.ppuaddr += 1;
        }
    }

    pub fn ppu_get_pattern_table(&self, background: bool) -> Chunks<'_, u8> {
        if background {
            let base_addr = if self.ppu_registers.ppuctrl.is_set(PPUCTRL::BPTNTABLE_ADDR) {
                0x1000
            } else {
                0x0000
            };
            self.cartridge.get_chr_rom()[base_addr..].chunks(16)
        } else {
            let base_addr = if self.ppu_registers.ppuctrl.is_set(PPUCTRL::SPTNTABLE_ADDR) {
                0x1000
            } else {
                0x0000
            };
            self.cartridge.get_chr_rom()[base_addr..].chunks(16)
        }
    }

    pub fn ppu_get_registers_mut(&mut self) -> &mut PPURegisters {
        &mut self.ppu_registers
    }

    pub fn ppu_get_registers(&self) -> &PPURegisters {
        &self.ppu_registers
    }

    pub fn ppu_read_nametable(&self, addr: usize) -> Result<u8, &'static str> {
        if addr < 0x2000 || addr >= 0x3000 {
            return Err("Invalid address lookup into nametable");
        } else {
            let nametable_mirror: Flags1::MIRRORING::Value = self
                .cartridge_header()
                .flags1
                .read_as_enum(Flags1::MIRRORING)
                .unwrap();

            return Ok(match nametable_mirror {
                Flags1::MIRRORING::Value::VERT => match addr {
                    0x2000..=0x23FF => self.ppu_ram[addr - 0x2000],
                    0x2400..=0x27FF => self.ppu_ram[0x400 + addr - 0x2400],
                    0x2800..=0x2BFF => self.ppu_ram[addr - 0x2800],
                    0x2C00..=0x2FFF => self.ppu_ram[0x400 + addr - 0x2C00],
                    _ => panic!("Should never happen"),
                },
                Flags1::MIRRORING::Value::HORZ => match addr {
                    0x2000..=0x23FF => self.ppu_ram[addr - 0x2000],
                    0x2400..=0x27FF => self.ppu_ram[addr - 0x2400],
                    0x2800..=0x2BFF => self.ppu_ram[0x400 + addr - 0x2800],
                    0x2C00..=0x2FFF => self.ppu_ram[0x400 + addr - 0x2C00],
                    _ => panic!("Should never happen"),
                },
            });
        }
    }

    pub fn ppu_get_nametable_base_addr(&self) -> usize {
        let name_table_addr: PPUCTRL::NTABLE_ADDR::Value = self
            .ppu_registers
            .ppuctrl
            .read_as_enum(PPUCTRL::NTABLE_ADDR)
            .unwrap();
        match name_table_addr {
            PPUCTRL::NTABLE_ADDR::Value::Addr2000 => 0x2000,
            PPUCTRL::NTABLE_ADDR::Value::Addr2400 => 0x2400,
            PPUCTRL::NTABLE_ADDR::Value::Addr2800 => 0x2800,
            PPUCTRL::NTABLE_ADDR::Value::Addr2C00 => 0x2C00,
        }
    }

    // TODO: This is a dumb hack
    pub fn translate_nametable_addr(&self, addr: u16) -> usize {
        let nametable_mirror: Flags1::MIRRORING::Value = self
            .cartridge_header()
            .flags1
            .read_as_enum(Flags1::MIRRORING)
            .unwrap();

        let bytes = u16::to_le_bytes(addr);

        match nametable_mirror {
            Flags1::MIRRORING::Value::VERT => match bytes[1] {
                (0x20..=0x23) => (addr - 0x2000) as usize,
                (0x24..=0x27) => (addr - 0x2400 + 0x400) as usize,
                (0x28..=0x2B) => (addr - 0x2800) as usize,
                (0x2C..=0x2F) => (addr - 0x2C00 + 0x400) as usize,
                _ => panic!(),
            },
            Flags1::MIRRORING::Value::HORZ => match bytes[1] {
                (0x20..=0x23) => (addr - 0x2000) as usize,
                (0x24..=0x27) => (addr - 0x2400) as usize,
                (0x28..=0x2B) => (addr - 0x2800 + 0x400) as usize,
                (0x2C..=0x2F) => (addr - 0x2C00 + 0x400) as usize,
                _ => panic!(),
            },
        }
    }
}
