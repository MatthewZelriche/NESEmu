//! Mapper000 - NROM-128 or NROM-256. The simplest mapper there is

use tock_registers::interfaces::Readable;

use super::{
    cartridge_data::{ines::Flags1, CartridgeData},
    Mapper, MirrorMode,
};

pub struct Mapper000 {
    cartridge_data: CartridgeData,
}

impl Mapper000 {
    pub fn new(cartridge_data: CartridgeData) -> Self {
        Self { cartridge_data }
    }
}

impl Mapper for Mapper000 {
    fn prg_read(&self, cpu_bus_address: usize) -> Result<u8, &'static str> {
        let internal_addr = match cpu_bus_address {
            (0x8000..=0xBFFF) => Ok(0x8000),
            (0xC000..=0xFFFF) => {
                // This mapper comes in two flavours: NROM-128 and NROM-256
                // The flavor can be inferred by the number of prg rom blocks
                // The NROM-128 flavor mirrors the first block for addresses in the
                // second block
                if self.cartridge_data.header.prg_rom_size > 1 {
                    Ok(0x8000)
                } else {
                    Ok(0xC000)
                }
            }
            _ => Err("Bad prg address read on cartridge"),
        };

        Ok(self.cartridge_data.get_prg_rom()[cpu_bus_address % internal_addr?])
    }

    fn prg_write(&mut self, _: usize, _: u8) -> Result<(), &'static str> {
        // Mapper zero means writing to prg rom is a no-op
        return Ok(());
    }

    fn chr_read(&self, ppu_bus_address: usize) -> Result<u8, &'static str> {
        match ppu_bus_address {
            0x0000..=0x1FFF => Ok(self.cartridge_data.get_chr_rom()[ppu_bus_address]),
            _ => Err("Bad chr address read on cartridge"),
        }
    }

    fn chr_read_pattern(&self, base_addr: usize, pattern_idx: u8) -> Option<&[u8]> {
        self.cartridge_data.get_chr_rom()[base_addr..]
            .chunks(16)
            .nth(pattern_idx as usize)
    }

    fn chr_write(&mut self, ppu_bus_address: usize, value: u8) -> Result<(), &'static str> {
        match ppu_bus_address {
            0x0000..=0x1FFF => {
                if let Some(ram) = self.cartridge_data.get_chr_ram() {
                    ram[ppu_bus_address] = value;
                }
            }
            _ => return Err("Bad CHR address write on cartridge"),
        }

        Ok(())
    }

    fn current_mirroring_mode(&self) -> MirrorMode {
        // Mapper 0 has a fixed mirroring mode
        match self
            .cartridge_data
            .header
            .flags1
            .read_as_enum(Flags1::MIRRORING)
            .unwrap()
        {
            Flags1::MIRRORING::Value::HORZ => MirrorMode::HORZ,
            Flags1::MIRRORING::Value::VERT => MirrorMode::VERT,
        }
    }
}
