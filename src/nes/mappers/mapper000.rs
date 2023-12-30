pub struct Mapper000 {
    prg_rom_size: u8,
}

use crate::nes::mappers::Mapper;

impl Mapper000 {
    pub fn new(prg_rom_size: u8) -> Self {
        Self { prg_rom_size }
    }
}

impl Mapper for Mapper000 {
    fn map_prg_address(&self, bus_address: usize) -> Result<usize, &'static str> {
        match bus_address {
            (0x8000..=0xBFFF) => Ok(bus_address % 0x8000),
            (0xC000..=0xFFFF) => {
                // This mapper comes in two flavours: NROM-128 and NROM-256
                // The flavor can be inferred by the number of prg rom blocks
                // The NROM-128 flavor mirrors the first block for addresses in the
                // second block
                let mirror = if self.prg_rom_size > 1 {
                    0x8000
                } else {
                    0xC000
                };
                Ok(bus_address % mirror)
            }
            _ => Err("Bad prg address read on cartridge"),
        }
    }

    fn write_register(&mut self, _: u16, _: u8) -> bool {
        return false;
    }
}
