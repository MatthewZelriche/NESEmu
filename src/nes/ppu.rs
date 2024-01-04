pub struct PPU {
    pub scanlines: usize,
    pub dots: usize,
}

impl PPU {
    const DOTS_PER_SCANLINE: usize = 341;
    const NUM_SCANLINES: usize = 262;
    pub fn new() -> Self {
        Self {
            scanlines: 0,
            dots: 21, // Simulates power-up delay
        }
    }

    pub fn step(&mut self) {
        self.dots += 1;
        if self.dots == PPU::DOTS_PER_SCANLINE {
            self.scanlines += 1;
            if self.scanlines >= PPU::NUM_SCANLINES {
                self.scanlines = 0;
            }
        }
        self.dots = self.dots % PPU::DOTS_PER_SCANLINE;
    }
}
