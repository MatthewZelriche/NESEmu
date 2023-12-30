pub trait Bus {
    fn read_byte(&self, address: usize) -> Result<u8, &str>;
    fn write_byte(&mut self, address: usize, value: u8) -> Result<(), &str>;
}

pub struct BusImpl {
    system_ram: [u8; 2048],
}

impl Bus for BusImpl {
    fn read_byte(&self, address: usize) -> Result<u8, &str> {
        match address {
            (0..=2048) => Ok(self.system_ram[address]),
            _ => Err("Bad address read on Bus"),
        }
    }

    fn write_byte(&mut self, address: usize, value: u8) -> Result<(), &str> {
        match address {
            (0..=2048) => Ok(self.system_ram[address] = value),
            _ => Err("Bad address write on Bus"),
        }
    }
}

impl Default for BusImpl {
    fn default() -> Self {
        Self {
            system_ram: [0u8; 2048],
        }
    }
}
