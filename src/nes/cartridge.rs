use core::slice;
use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Seek, SeekFrom},
};

use tock_registers::interfaces::Readable;

use super::ines::{Flags1, INESHeader};

pub struct Cartridge {
    header: INESHeader,
    trainer: Option<[u8; 512]>,
}

impl Cartridge {
    const VALID_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
    const HEADER_SIZE: u8 = 16;
    const PRG_ROM_BLOCK_SZ: u16 = 16384;
    const CHR_ROM_BLOCK_SZ: u16 = 8192;

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
        file.read_exact(slice::from_mut(&mut header.flags1.get()))?;
        file.read_exact(slice::from_mut(&mut header.flags2.get()))?;
        file.read_exact(slice::from_mut(&mut header.prg_ram_size))?;
        file.read_exact(slice::from_mut(&mut header.tv_system))?;
        // Don't support CHR RAM currently
        if header.chr_rom_size == 0 {
            return Err(Error::from(ErrorKind::Unsupported));
        }
        // Skip the rest of the header
        file.seek(SeekFrom::Start(Cartridge::HEADER_SIZE.into()))?;
        // read trainer, if it exists
        let mut trainer = None;
        if header.flags1.is_set(Flags1::HAS_TRAINER) {
            let mut trainer_data = [0u8; 512];
            file.read_exact(&mut trainer_data)?;
            trainer = Some(trainer_data);
        }

        Ok(Self { header, trainer })
    }
}

#[cfg(test)]
mod tests {
    use tock_registers::interfaces::Readable;

    use super::Cartridge;

    #[test]
    fn load_nestest() {
        let cartridge = Cartridge::new("res/nestest.nes").expect("Failed to construct cartridge");
        // Validate header correct
        assert_eq!(cartridge.header.prg_rom_size, 1);
        assert_eq!(cartridge.header.chr_rom_size, 1);
        assert_eq!(cartridge.header.flags1.get(), 0);
        assert_eq!(cartridge.header.flags2.get(), 0);
        assert_eq!(cartridge.header.prg_ram_size, 0);
        assert_eq!(cartridge.header.tv_system, 0);
        // Validate no trainer
        assert_eq!(cartridge.trainer, None);
    }
}
