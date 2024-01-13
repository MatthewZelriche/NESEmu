use bitfield::{Bit, BitMut, BitRange, BitRangeMut};
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
            bus.ppu_get_registers_mut()
                .ppustatus
                .modify(PPUSTATUS::VBLANK::SET);
            self.generated_interrupt = true
                && bus
                    .ppu_get_registers_mut()
                    .ppuctrl
                    .is_set(PPUCTRL::NMI_ENABLE);
        } else if self.scanlines == 261 && self.dots == 1 {
            // Pre-render scanline...
            bus.ppu_get_registers_mut()
                .ppustatus
                .modify(PPUSTATUS::VBLANK::CLEAR);
        }
        return false;
    }

    pub fn draw_to_framebuffer<T: FrameBuffer>(&mut self, fb: &mut T, bus: &Bus) {
        // Compute the start index into the current nametable
        let start_fine_x = bus.ppu_get_registers().fine_x as u16;
        let start_fine_y = bus.ppu_get_registers().fine_y as u16;
        let nametable_start_idx = (((start_fine_y / 8) * 32) + (start_fine_x / 8)) as usize;

        // Compute the start address from the start index
        let mut nametable_addr = (bus.ppu_get_nametable_base_addr() + nametable_start_idx) as u16;

        let mut curr_px_x = 0;
        let mut curr_px_y = 0;
        let fine_offset_x = (start_fine_x % 8) as usize;
        let fine_offset_y = (start_fine_y % 8) as usize;
        for _ in 0..960 + 32 {
            let tile_idx = bus.ppu_read_nametable(nametable_addr as usize).unwrap();
            // This monstrosity taken from https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
            let attrib_table_addr = 0x23C0
                | (nametable_addr & 0x0C00)
                | ((nametable_addr >> 4) & 0x38)
                | ((nametable_addr >> 2) & 0x07);
            let attrib_table_val = bus.ppu_read_nametable(attrib_table_addr as usize).unwrap();
            let coarse_x = (nametable_addr as u16).bit_range(4, 0);
            let coarse_y = (nametable_addr as u16).bit_range(9, 5);
            let palette_idx = self.compute_palette_index(attrib_table_val, coarse_x, coarse_y);
            self.plot_tile(
                curr_px_x,
                curr_px_y,
                fine_offset_x,
                fine_offset_y,
                tile_idx,
                palette_idx,
                &bus,
                fb,
            );

            // Update address for next tile render
            curr_px_x += 8;
            if coarse_x == 31 {
                nametable_addr.set_bit_range(4, 0, 0); // Reset coarse x to zero
                                                       // Flip bit to switch horz nametable
                nametable_addr.set_bit(10, !(nametable_addr as u16).bit(10));
            } else {
                //nametable_addr += 1; // increment course x
                nametable_addr.set_bit_range(4, 0, coarse_x + 1);
            }
            if curr_px_x > 8 * 31 {
                curr_px_y += 8;
                curr_px_x = 0;

                if coarse_y == 29 {
                    nametable_addr.set_bit_range(9, 5, 0); // Reset coarse y to zero
                                                           // Flip bit to switch vert nametable
                    nametable_addr.set_bit(11, !(nametable_addr as u16).bit(11));
                } else if coarse_y == 31 {
                    nametable_addr.set_bit_range(9, 5, 0); // Reset coarse y to zero
                } else {
                    nametable_addr.set_bit_range(9, 5, coarse_y + 1);
                }
            }
        }
    }

    pub fn compute_palette_index(&self, attrib_value: u8, coarse_x: u8, coarse_y: u8) -> u8 {
        // The second bit of our tile coordinates contains the information
        // we need to determine our quadrant
        match (coarse_y.bit(1), coarse_x.bit(1)) {
            (false, false) => attrib_value.bit_range(1, 0),
            (false, true) => attrib_value.bit_range(3, 2),
            (true, false) => attrib_value.bit_range(5, 4),
            (true, true) => attrib_value.bit_range(7, 6),
        }
    }

    fn plot_tile<T: FrameBuffer>(
        &self,
        tile_px_x: usize,
        tile_px_y: usize,
        fine_offset_x: usize,
        fine_offset_y: usize,
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
                fb.plot_pixel(x - fine_offset_x, y - fine_offset_y, color);
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
