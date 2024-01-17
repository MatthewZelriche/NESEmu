use core::slice;
use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Seek, SeekFrom},
};

use tock_registers::interfaces::{Readable, Writeable};

use super::{
    ines::{Flags1, Flags2, INESHeader},
    mappers::{get_mapper, Mapper},
};

enum CHR {
    ROM(Vec<u8>),
    RAM(Vec<u8>),
}

pub struct Cartridge {
    header: INESHeader,
    _trainer: Option<[u8; 512]>,
    prg_rom: Vec<u8>,
    chr_data: CHR,
    pub mapper: Box<dyn Mapper>,
}

impl Cartridge {
    const VALID_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
    const HEADER_SIZE: u8 = 16;
    const PRG_ROM_BLOCK_SZ: usize = 16384;
    const CHR_ROM_BLOCK_SZ: usize = 8192;

    pub fn new(path: &str) -> Result<Self, Error> {
        // Open the ROM file
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(0))?;
        // Validate the magic number string
        let mut magic = [0u8; Cartridge::VALID_MAGIC.len()];
        file.read_exact(&mut magic)?;
        if magic != Cartridge::VALID_MAGIC {
            return Err(Error::from(ErrorKind::InvalidInput));
        }
        // Read in header data
        let mut header = INESHeader::default();
        file.read_exact(slice::from_mut(&mut header.prg_rom_size))?;
        file.read_exact(slice::from_mut(&mut header.chr_rom_size))?;
        let mut flags1 = 0;
        file.read_exact(slice::from_mut(&mut flags1))?;
        header.flags1.set(flags1);
        let mut flags2 = 0;
        file.read_exact(slice::from_mut(&mut flags2))?;
        header.flags2.set(flags2);
        file.read_exact(slice::from_mut(&mut header.prg_ram_size))?;
        file.read_exact(slice::from_mut(&mut header.tv_system))?;
        // Skip the rest of the header
        file.seek(SeekFrom::Start(Cartridge::HEADER_SIZE.into()))?;
        // read trainer, if it exists
        let mut _trainer = None;
        if header.flags1.is_set(Flags1::HAS_TRAINER) {
            let mut trainer_data = [0u8; 512];
            file.read_exact(&mut trainer_data)?;
            _trainer = Some(trainer_data);
        }
        // Read PRG ROM
        let mut prg_rom = Vec::new();
        prg_rom.resize(
            header.prg_rom_size as usize * Cartridge::PRG_ROM_BLOCK_SZ,
            0u8,
        );
        file.read_exact(&mut prg_rom)?;

        let chr_data = if header.chr_rom_size != 0 {
            let mut chr_rom = Vec::new();
            // Read CHR ROM
            chr_rom.resize(
                header.chr_rom_size as usize * Cartridge::CHR_ROM_BLOCK_SZ,
                0u8,
            );
            file.read_exact(&mut chr_rom)?;
            CHR::ROM(chr_rom)
        } else {
            let mut chr_ram = Vec::new();
            chr_ram.resize(Cartridge::CHR_ROM_BLOCK_SZ, 0);
            CHR::RAM(chr_ram)
        };

        let prg_rom_size = header.prg_rom_size;
        let mapper_id = header.flags1.read(Flags1::MAPPER_LOWER)
            + (header.flags2.read(Flags2::MAPPER_UPPER) << 4);
        Ok(Self {
            header,
            _trainer,
            prg_rom,
            chr_data,
            mapper: get_mapper(mapper_id, prg_rom_size)
                .map_err(|_| Error::from(ErrorKind::Unsupported))?,
        })
    }

    pub fn get_prg_rom(&self) -> &[u8] {
        &self.prg_rom
    }

    pub fn get_chr_ram(&mut self) -> Option<&mut [u8]> {
        match &mut self.chr_data {
            CHR::ROM(_) => None,
            CHR::RAM(data) => Some(data),
        }
    }

    pub fn get_chr_rom(&self) -> &[u8] {
        match &self.chr_data {
            CHR::ROM(data) | CHR::RAM(data) => data,
        }
    }

    pub fn get_header(&self) -> &INESHeader {
        &self.header
    }
}

#[cfg(test)]
mod tests {
    use tock_registers::interfaces::Readable;

    use super::Cartridge;

    #[test]
    fn load_nestest() {
        let mut cartridge =
            Cartridge::new("res/nestest.nes").expect("Failed to construct cartridge");
        // Validate header correct
        assert_eq!(cartridge.header.prg_rom_size, 1);
        assert_eq!(cartridge.header.chr_rom_size, 1);
        assert_eq!(cartridge.header.flags1.get(), 0);
        assert_eq!(cartridge.header.flags2.get(), 0);
        assert_eq!(cartridge.header.prg_ram_size, 0);
        assert_eq!(cartridge.header.tv_system, 0);
        // Validate no trainer
        assert_eq!(cartridge.trainer, None);

        // Check sizes of buffers
        assert_eq!(cartridge.prg_rom.len(), Cartridge::PRG_ROM_BLOCK_SZ);
        assert_eq!(cartridge.chr_rom.len(), Cartridge::CHR_ROM_BLOCK_SZ);

        // Check first and last few bytes of prg rom
        assert_eq!(cartridge.prg_rom[..4], [0x4c, 0xF5, 0xC5, 0x60]);
        assert_eq!(*cartridge.prg_rom.last().unwrap(), 0xC5);

        // Validate mapper is working correctly
        assert_eq!(cartridge.mapper.map_prg_address(0xFFFC).unwrap(), 0x3FFC);
        assert_eq!(cartridge.mapper.map_prg_address(0xC000).unwrap(), 0x0000);
        assert_eq!(cartridge.mapper.write_register(0xC000, 0), Ok(()));
    }
}
