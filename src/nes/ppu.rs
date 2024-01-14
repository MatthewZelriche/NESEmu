use bitfield::{Bit, BitMut, BitRange, BitRangeMut};
use tock_registers::interfaces::{ReadWriteable, Readable};

use super::{
    bus::Bus,
    ppu_registers::{PPUCTRL, PPUSTATUS},
    screen::FrameBuffer,
};

pub struct PPU {
    nametable_addr: u16,
    x_scroll: u8,
    y_scroll: u8,
    pub scanlines: usize,
    pub dots: usize,
    generated_interrupt: bool,
}

impl PPU {
    const DOTS_PER_SCANLINE: usize = 341;
    const NUM_SCANLINES: usize = 262;
    pub fn new() -> Self {
        Self {
            nametable_addr: 0x2000,
            x_scroll: 0,
            y_scroll: 0,
            scanlines: 0,
            dots: 21, // Simulates power-up delay
            generated_interrupt: false,
        }
    }

    pub fn step<T: FrameBuffer>(&mut self, fb: &mut T, bus: &mut Bus) -> bool {
        // Each step processes a single dot/pixel
        // Though in reality we don't render under the scanline is finished
        self.dots += 1;

        if self.dots == PPU::DOTS_PER_SCANLINE {
            // We just completed a scanline, render it
            // Don't bother drawing to the overdraw scanlines, they will never be seen anyway
            if self.scanlines <= 239 {
                self.draw_scanline(fb, bus);
            }
            self.scanlines += 1;
            self.dots = 0;

            if self.scanlines >= PPU::NUM_SCANLINES {
                // We just finished a frame
                self.prepare_next_frame(bus);
                return true;
            }
        }

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

    fn prepare_next_frame(&mut self, bus: &mut Bus) {
        self.scanlines = 0;
        // Update x_scroll and y_scroll
        // My understanding is that programs are meant to change these only during vblank,
        // so it should be safe to check them only once per frame
        self.y_scroll = bus.ppu_get_registers().fine_y;
        self.x_scroll = bus.ppu_get_registers().fine_x;

        // Reconstruct the starting address of the nametable based on PPUSCROLL
        let nametable_start_idx =
            ((((self.y_scroll as usize) / 8) * 32) + (self.x_scroll as usize / 8)) as usize;
        self.nametable_addr = (bus.ppu_get_nametable_base_addr() + nametable_start_idx) as u16;
    }

    // TODO: Handle sprite rendering
    pub fn draw_scanline<T: FrameBuffer>(&mut self, fb: &mut T, bus: &mut Bus) {
        let pixel_space_y = self.scanlines;

        // x and y scroll represent the nametable pixel coordinate that is to be situated at the top-left
        // corner of the screen.
        // However, we also need "wrapped" versions of these coordinates which represent offsets into an
        // individual 8x8 pixel nametable entry
        let mut fine_x_wrapped = self.x_scroll % 8;
        let fine_y_wrapped = self.y_scroll % 8;

        let coarse_y = (self.nametable_addr as u16).bit_range(9, 5);

        for pixel_space_x in 0..256 {
            // Compute pattern table idx and palette idx
            // This monstrosity taken from https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
            let attrib_table_addr = 0x23C0
                | (self.nametable_addr & 0x0C00)
                | ((self.nametable_addr >> 4) & 0x38)
                | ((self.nametable_addr >> 2) & 0x07);
            let attrib_table_val = bus.ppu_read_nametable(attrib_table_addr as usize).unwrap();
            let coarse_x = (self.nametable_addr as u16).bit_range(4, 0);
            let pt_idx = bus
                .ppu_read_nametable(self.nametable_addr as usize)
                .unwrap();
            let palette_num = self.compute_palette_index(attrib_table_val, coarse_x, coarse_y);

            // Get the chr tile data, a 16 byte chunk representing an individual 8x8 tile
            let mut pattern_table = bus.ppu_get_pattern_table(true);
            let tile = pattern_table.nth(pt_idx as usize).unwrap();
            // Compute palette color
            // Read the palette idx data from both bitplanes in the tile
            let bit_idx = 7 - (fine_x_wrapped); // Flip the bit index so we go from left to right over the bits
            let y_tile_idx = (fine_y_wrapped as usize + pixel_space_y) % 8;
            let low_bit = u8::from(tile[y_tile_idx as usize].bit(bit_idx as usize));
            let high_bit = u8::from(tile[(y_tile_idx + 8) as usize].bit(bit_idx as usize));
            let palette_idx = low_bit + (high_bit << 1);
            let color = bus
                .palette_memory
                .get_color_by_idx(palette_num, palette_idx)
                .unwrap();

            // Write pixel color into the fb
            fb.plot_pixel(pixel_space_x, pixel_space_y, color);

            // Handle offset x wrapping into the nametable entry
            fine_x_wrapped += 1;
            if fine_x_wrapped > 7 {
                fine_x_wrapped = 0;

                // Increment Coarse X
                if coarse_x == 31 {
                    self.nametable_addr.set_bit_range(4, 0, 0); // Wrap Coarse X to zero
                                                                // Flip bit to switch horz nametable
                    self.nametable_addr
                        .set_bit(10, !(self.nametable_addr as u16).bit(10));
                } else {
                    self.nametable_addr.set_bit_range(4, 0, coarse_x + 1);
                }
            }
        }

        // If our y coordinate is about to enter a new nametable entry...
        if (self.y_scroll as usize + pixel_space_y) % 8 == 7 {
            // Increment Coarse Y
            if coarse_y == 29 {
                self.nametable_addr.set_bit_range(9, 5, 0); // Wrap coarse y to zero
                                                            // Flip bit to switch vert nametable
                self.nametable_addr
                    .set_bit(11, !(self.nametable_addr as u16).bit(11));
            } else if coarse_y == 31 {
                self.nametable_addr.set_bit_range(9, 5, 0);
            } else {
                self.nametable_addr.set_bit_range(9, 5, coarse_y + 1);
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

    pub fn generated_interrupt(&mut self) -> bool {
        let res = self.generated_interrupt;
        if self.generated_interrupt {
            self.generated_interrupt = false;
        }

        res
    }
}
