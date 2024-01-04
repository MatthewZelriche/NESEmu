use bitfield::Bit;
use eframe::epaint::Color32;

use super::{bus::Bus, screen::FrameBuffer};

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

    pub fn step<T: FrameBuffer>(&mut self, bus: &mut Bus, fb: &mut T) {
        self.dots += 1;
        if self.dots == PPU::DOTS_PER_SCANLINE {
            self.scanlines += 1;
            if self.scanlines >= PPU::NUM_SCANLINES {
                self.scanlines = 0;
            }
        }
        self.dots = self.dots % PPU::DOTS_PER_SCANLINE;

        self.plot_tile(0, 0, 6, bus, fb);
    }

    fn plot_tile<T: FrameBuffer>(
        &self,
        tile_x: usize,
        tile_y: usize,
        pt_idx: u8,
        bus: &mut Bus,
        fb: &mut T,
    ) {
        // Get the pattern table, split into 16 byte chunks representing individual 8x8 tiles
        let mut pattern_table = bus.ppu_get_pattern_table();
        let tile = pattern_table.nth(pt_idx as usize).unwrap();

        // Draw the tile to the framebuffer
        // TODO: Draw at specified tile offset
        for y in 0..8 {
            for x in 0..8 {
                // Read the palette idx data from both bitplanes in the tile
                let bit_idx = 7 - x; // Flip the bit index so we go from left to right over the bits
                let palette_idx: u8 =
                    u8::from(tile[y].bit(bit_idx)) + u8::from(tile[y + 8].bit(bit_idx));

                // TODO: Proper palette index lookup
                fb.plot_pixel(
                    x,
                    y,
                    match palette_idx {
                        0 => Color32::BLACK,
                        1 => Color32::WHITE,
                        2 => Color32::BLUE,
                        3 => Color32::RED,
                        _ => panic!(),
                    },
                );
            }
        }
    }
}
