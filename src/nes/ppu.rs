use bitfield::Bit;
use eframe::epaint::Color32;
use tock_registers::interfaces::ReadWriteable;

use super::{bus::Bus, ppu_registers::PPUSTATUS, screen::FrameBuffer};

pub struct PPU {
    pub scanlines: usize,
    pub dots: usize,
    generated_interrupt: bool,
}

impl PPU {
    const DOTS_PER_SCANLINE: usize = 341;
    const NUM_SCANLINES: usize = 262;
    pub fn new() -> Self {
        Self {
            scanlines: 0,
            dots: 21, // Simulates power-up delay
            generated_interrupt: false,
        }
    }

    pub fn step(&mut self, bus: &mut Bus) -> bool {
        self.dots += 1;
        if self.dots == PPU::DOTS_PER_SCANLINE {
            self.scanlines += 1;
            if self.scanlines >= PPU::NUM_SCANLINES {
                self.scanlines = 0;
                // We just finished a frame
                return true;
            }
        }
        self.dots = self.dots % PPU::DOTS_PER_SCANLINE;

        // Handle vblank
        if self.scanlines == 241 && self.dots == 1 {
            bus.ppu_get_registers()
                .ppustatus
                .modify(PPUSTATUS::VBLANK::SET);
            self.generated_interrupt = true;
        } else if self.scanlines == 261 && self.dots == 1 {
            // Pre-render scanline...
            bus.ppu_get_registers()
                .ppustatus
                .modify(PPUSTATUS::VBLANK::CLEAR);
        }
        return false;
    }

    pub fn draw_to_framebuffer<T: FrameBuffer>(&self, fb: &mut T, bus: &Bus) {
        let name_table = bus.ppu_get_nametable();
        for i in 0..960 {
            let tile_px_x = (i % 32) * 8;
            let tile_px_y = (i / 30) * 8;
            let tile_idx = name_table[i];
            self.plot_tile(tile_px_x, tile_px_y, tile_idx, &bus, fb);
        }
    }

    // TODO: This can be removed at some point when we are done displaying full pattern tables
    pub fn debug_render<T: FrameBuffer>(&self, bus: &Bus, fb: &mut T) {
        // Render the entire pattern table
        for i in 0..256 {
            let tile_px_x = (i % 31) * 8;
            let tile_px_y = (i / 31) * 8;
            self.plot_tile(tile_px_x, tile_px_y, i.try_into().unwrap(), bus, fb);
        }
    }

    fn plot_tile<T: FrameBuffer>(
        &self,
        tile_px_x: usize,
        tile_px_y: usize,
        pt_idx: u8,
        bus: &Bus,
        fb: &mut T,
    ) {
        // Get the pattern table, split into 16 byte chunks representing individual 8x8 tiles
        // TODO: Handle sprite rendering
        let mut pattern_table = bus.ppu_get_pattern_table(true);
        let tile = pattern_table.nth(pt_idx as usize).unwrap();

        // Draw the tile to the framebuffer
        for y in tile_px_y..tile_px_y + 8 {
            for x in tile_px_x..tile_px_x + 8 {
                // Read the palette idx data from both bitplanes in the tile
                let bit_idx = 7 - (x % 8); // Flip the bit index so we go from left to right over the bits
                let y_tile_idx = y % 8;
                let palette_idx: u8 = u8::from(tile[y_tile_idx].bit(bit_idx))
                    + u8::from(tile[y_tile_idx + 8].bit(bit_idx));

                // TODO: Proper palette index lookup
                fb.plot_pixel(
                    x,
                    y,
                    match palette_idx {
                        0 => Color32::BLACK,
                        1 => Color32::WHITE,
                        2 => Color32::LIGHT_GRAY,
                        3 => Color32::DARK_GRAY,
                        _ => panic!(),
                    },
                );
            }
        }
    }

    pub fn generated_interrupt(&mut self) -> bool {
        let res = self.generated_interrupt;
        if self.generated_interrupt {
            self.generated_interrupt = false;
        }

        res
    }
}
