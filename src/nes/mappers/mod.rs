//! Early cartridges were very primitive. Throughout the console's lifetime, different cartridge hardware
//! designs were developed to overcome certain limitations, such as the 16-bit address space and the lack of
//! scrolling in multiple screen directions. Mappers represent the equivalence classes of different cartridge
//! hardware designs. They sit in front of the cartridge data itself, intercepting and potentially modifying
//! read and write requests to the data. Write requests to otherwise unwriteable ROM addresses can be
//! interpreted as commands for the mapper to configure itself in a certain way.

use std::io::{Error, ErrorKind};

use self::{cartridge_data::CartridgeData, mapper000::Mapper000};

mod cartridge_data;
mod mapper000;

pub enum MirrorMode {
    HORZ,
    VERT,
}

pub trait Mapper {
    /// Read a single byte of data from the cartridge's PRG data
    ///
    /// The CPU bus maps PRG data to addresses 0x4020 - 0xFFFF, so calling this function with bus addresses
    /// outside this range is guarunteed to fail. In addition, not all Mappers actually map PRG data to this
    /// entire range of addresses, so addresses within this range are not guarunteed to succeed either.
    fn prg_read(&self, cpu_bus_address: usize) -> Result<u8, &'static str>;
    /// Write a single byte of data to the cartridge's PRG data
    ///
    /// PRG data consists of either ROM or RAM. Calling this function on a PRG RAM address works as expected,
    /// but calling it on a PRG ROM address will either result in a no-op or it may activate a mapper register,
    /// depending on the mapper used.
    fn prg_write(&mut self, cpu_bus_address: usize, val: u8) -> Result<(), &'static str>;

    /// Reads a single byte of data from the cartridge's CHR data
    ///
    /// The PPU bus maps CHR data from 0x0000 - 0x1FFF, so calling this function with bus addresses outside
    /// this range is guarunteed to fail.
    fn chr_read(&self, ppu_bus_address: usize) -> Result<u8, &'static str>;
    /// Reads a single pattern entry (consisting of 16 bytes) from the cartridge's CHR data
    ///
    /// This function returns None if the pattern idx is out of range. Note that no checks are performed to
    /// confirm the returned bytes are a valid chr pattern. An invalid pattern could be returned if CHR RAM
    /// is being read and the program wrote non-pattern data to it, then later retrieved it as if it were a
    /// pattern.
    fn chr_read_pattern(&self, base_addr: usize, pattern_idx: u8) -> Option<&[u8]>;
    /// Writes a single byte of data to the cartridge's CHR data
    ///
    /// This is a no-op unless the cartridge supports CHR RAM. The PPU bus maps CHR data from 0x0000 - 0x1FFF,
    /// so calling this function with bus addresses outside this range is guarunteed to fail.
    fn chr_write(&mut self, ppu_bus_address: usize, value: u8) -> Result<(), &'static str>;

    /// Gets the current nametable mirroring mode for this cartridge.
    ///
    /// Some mappers support programmatically switching the nametable mirroring mode  at runtime. If a mapper
    /// does not support this behavior, then this function will return whatever hardcoded mirroring mode was
    /// stored in the iNES header.
    fn current_mirroring_mode(&self) -> MirrorMode;
}

/// Creates a new mapper from a given ROM file
///
/// Fails if the rom's specified mapper is not supported, or if there is a problem reading the rom file.
pub fn new_mapper(rom_path: &str) -> Result<Box<dyn Mapper>, Error> {
    let cartridge_data = CartridgeData::new(rom_path)?;
    match cartridge_data.mapper_id {
        0 => Ok(Box::new(Mapper000::new(cartridge_data))),
        _ => Err(Error::from(ErrorKind::Unsupported)),
    }
}
