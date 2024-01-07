use bitfield::{Bit, BitRange};
use tock_registers::interfaces::{ReadWriteable, Readable};

use super::{
    bus::Bus,
    ppu_registers::{PPUCTRL, PPUSTATUS},
    screen::FrameBuffer,
};

pub struct PPU {
    pub scanlines: usize,
    pub dots: usize,
    generated_interrupt: bool,
    tile_x: u8,
    tile_y: u8,
}

impl PPU {
    const DOTS_PER_SCANLINE: usize = 341;
    const NUM_SCANLINES: usize = 262;
    pub fn new() -> Self {
        Self {
            scanlines: 0,
            dots: 21, // Simulates power-up delay
            generated_interrupt: false,
            tile_x: 0,
            tile_y: 0,
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
            self.generated_interrupt =
                true && bus.ppu_get_registers().ppuctrl.is_set(PPUCTRL::NMI_ENABLE);
        } else if self.scanlines == 261 && self.dots == 1 {
            // Pre-render scanline...
            bus.ppu_get_registers()
                .ppustatus
                .modify(PPUSTATUS::VBLANK::CLEAR);
        }
        return false;
    }

    pub fn draw_to_framebuffer<T: FrameBuffer>(&mut self, fb: &mut T, bus: &Bus) {
        let name_table = bus.ppu_get_nametable();
        let attribute_table = &name_table[960..1024];
        for i in 0..960 {
            self.tile_x = (i as u16).bit_range(4, 0);
            self.tile_y = (i as u16).bit_range(9, 5);
            let tile_px_x = (i % 32) * 8;
            let tile_px_y = (i / 32) * 8;
            let tile_idx = name_table[i];
            let palette_idx = self.compute_palette_index(attribute_table);
            self.plot_tile(tile_px_x, tile_px_y, tile_idx, palette_idx, &bus, fb);
        }
    }

    pub fn compute_palette_index(&self, attribute_table: &[u8]) -> u8 {
        // Through the magic of power-of-two numbers, we can discern the block
        // coordinates and the block quadrant coordinates just by examining the bits of
        // our tile x,y coordinates
        let block_x: u8 = self.tile_x.bit_range(4, 2);
        let block_y: u8 = self.tile_y.bit_range(4, 2);
        let block_val = attribute_table[(block_y as usize * 8) + block_x as usize];

        // The second bit of our tile coordinates contains the information
        // we need to determine our quadrant
        match (self.tile_y.bit(1), self.tile_x.bit(1)) {
            (false, false) => block_val.bit_range(1, 0),
            (false, true) => block_val.bit_range(3, 2),
            (true, false) => block_val.bit_range(5, 4),
            (true, true) => block_val.bit_range(7, 6),
        }
    }

    fn plot_tile<T: FrameBuffer>(
        &self,
        tile_px_x: usize,
        tile_px_y: usize,
        pt_idx: u8,
        palette_num: u8,
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
                let low_bit = u8::from(tile[y_tile_idx].bit(bit_idx));
                let high_bit = u8::from(tile[y_tile_idx + 8].bit(bit_idx));
                let palette_idx = low_bit + (high_bit << 1);

                let color = bus
                    .palette_memory
                    .get_color_by_idx(palette_num, palette_idx)
                    .unwrap();
                fb.plot_pixel(x, y, color);
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
