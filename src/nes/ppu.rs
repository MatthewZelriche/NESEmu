pub struct PPU {
    scanlines: u16,
    cycles: usize,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            scanlines: 0,
            cycles: 21, // Simulates power-up delay
        }
    }

    pub fn step(&mut self) {
        if self.cycles == 341 {
            self.cycles = 0;
            self.scanlines += 1;
        } else {
            self.cycles += 1;
        }
    }
}
