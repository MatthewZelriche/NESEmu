use std::io::Error;

use super::cartridge::Cartridge;

pub trait Bus {
    fn read_byte(&self, address: usize) -> Result<u8, &'static str>;
    fn read_exact(&self, address: usize, buf: &mut [u8]) -> Result<(), &'static str>;
    fn write_byte(&mut self, address: usize, value: u8) -> Result<(), &'static str>;
}

pub struct BusImpl {
    cartridge: Cartridge,
    system_ram: [u8; 2048],
}

impl BusImpl {
    pub fn new(rom_path: &str) -> Result<Self, Error> {
        Ok(Self {
            cartridge: Cartridge::new(rom_path)?,
            system_ram: [0u8; 2048], // Real RAM starts in an uninit state, but rust
                                     // makes us init it
        })
    }
}

impl Bus for BusImpl {
    fn read_byte(&self, address: usize) -> Result<u8, &'static str> {
        match address {
            (0..=0x1FFF) => Ok(self.system_ram[address % 0x0800]),
            (0x4020..=0xFFFF) => {
                let prg_addr = self.cartridge.mapper.map_prg_address(address)?;
                Ok(self.cartridge.get_prg_rom()[prg_addr])
            }
            _ => Err("Bad address read on Bus"),
        }
    }

    fn read_exact(&self, address: usize, buf: &mut [u8]) -> Result<(), &'static str> {
        let len = buf.len();
        for i in 0..len {
            buf[i] = self.read_byte(address + i)?;
        }
        Ok(())
    }

    fn write_byte(&mut self, address: usize, value: u8) -> Result<(), &'static str> {
        match address {
            (0..=2048) => Ok(self.system_ram[address] = value),
            _ => Err("Bad address write on Bus"),
        }
    }
}
