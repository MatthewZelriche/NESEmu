use std::io::Error;

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
    registers::InMemoryRegister,
};

use super::cartridge::Cartridge;

register_bitfields!(
    u8,
    pub PPUCTRL [
        NTABLE_ADDR         OFFSET(0) NUMBITS(2) [
            Addr2000 = 0,
            Addr2400 = 1,
            Addr2800 = 2,
            Addr2C00 = 3,
        ],
        VRAM_INC            OFFSET(2) NUMBITS(1) [],
        SPTNTABLE_ADDR      OFFSET(3) NUMBITS(1) [],
        BPTNTABLE_ADDR      OFFSET(4) NUMBITS(1) [],
        SPRITE_SIZE         OFFSET(5) NUMBITS(1) [],
        MASTER_SLAVE_SELECT OFFSET(6) NUMBITS(1) [],
        NMI_ENABLE          OFFSET(7) NUMBITS(1) [],
    ]
);

register_bitfields!(
    u8,
    pub PPUSTATUS [
        UNUSED              OFFSET(0) NUMBITS(5) [],
        SPRITE_OVERFLOW     OFFSET(5) NUMBITS(1) [],
        SPRITE0_HIT         OFFSET(6) NUMBITS(1) [],
        VBLANK              OFFSET(7) NUMBITS(1) [],
    ]
);

pub struct PPURegisters {
    ppuctrl: InMemoryRegister<u8, PPUCTRL::Register>,
    ppustatus: InMemoryRegister<u8, PPUSTATUS::Register>,
}

impl Default for PPURegisters {
    fn default() -> Self {
        Self {
            ppuctrl: InMemoryRegister::new(0),
            ppustatus: InMemoryRegister::new(0),
        }
    }
}

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
    pub fn cpu_read_byte(&self, address: usize) -> Result<u8, &'static str> {
        match address {
            (0..=0x1FFF) => Ok(self.cpu_ram[address % 0x0800]),
            (0x2000..=0x3FFF) => self.read_ppu_register(address),
            (0x4020..=0xFFFF) => {
                let prg_addr = self.cartridge.mapper.map_prg_address(address)?;
                Ok(self.cartridge.get_prg_rom()[prg_addr])
            }
            _ => Err("Bad address read on Bus"),
        }
    }

    pub fn cpu_read_exact(&self, address: usize, buf: &mut [u8]) -> Result<(), &'static str> {
        let len = buf.len();
        for i in 0..len {
            buf[i] = self.cpu_read_byte(address + i)?;
        }
        Ok(())
    }

    pub fn cpu_write_byte(&mut self, address: usize, value: u8) -> Result<(), &'static str> {
        match address {
            (0..=2048) => Ok(self.cpu_ram[address] = value),
            (0x2000..=0x3FFF) => self.write_ppu_register(address, value),
            _ => Err("Bad address write on Bus"),
        }
    }

    pub fn read_ppu_register(&self, address: usize) -> Result<u8, &'static str> {
        match address {
            0x2000 => Ok(self.ppu_registers.ppuctrl.get()),
            0x2001 => todo!(),
            0x2002 => Ok(self.ppu_registers.ppustatus.get()),
            (0x2003..=0x2007) => todo!(),
            _ => Err("Bad Read on PPU register"),
        }
    }

    pub fn write_ppu_register(&mut self, address: usize, value: u8) -> Result<(), &'static str> {
        match address {
            0x2000 => Ok(self.ppu_registers.ppuctrl.set(value)),
            0x2001 => todo!(),
            0x2002 => Ok(self.ppu_registers.ppustatus.set(value)),
            (0x2003..=0x2007) => todo!(),
            _ => Err("Bad Write on PPU Register"),
        }
    }
}
