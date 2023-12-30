use self::mapper000::Mapper000;

mod mapper000;

pub trait Mapper {
    fn map_prg_address(&self, bus_address: usize) -> Result<usize, ()>;
    fn write_register(&mut self, address: u16, val: u8) -> bool;
}

pub fn get_mapper(mmc_id: u16, prg_rom_size: u8) -> Box<dyn Mapper> {
    match mmc_id {
        0 => Box::new(Mapper000::new(prg_rom_size)),
        _ => unimplemented!(),
    }
}
