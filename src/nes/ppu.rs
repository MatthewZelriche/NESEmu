use bitfield::{Bit, BitMut, BitRange, BitRangeMut};
use tock_registers::{
    interfaces::{ReadWriteable, Readable},
    register_bitfields,
    registers::InMemoryRegister,
};

use super::{
    bus::Bus,
    ppu_registers::{PPUCTRL, PPUSTATUS},
    screen::FrameBuffer,
};

// TODO:
// horizontal scrolling is broken
// Max 8 Sprites per line (+ sprite overflow)
// 8x16 bit sprite mode

register_bitfields! [
    u8,
    SpriteAttribs [
        PALETTE           OFFSET(0) NUMBITS(2) [],
        UNIMPLEMENTED     OFFSET(2) NUMBITS(3) [],
        PRIORITY          OFFSET(5) NUMBITS(1) [],
        FLIP_HORZ         OFFSET(6) NUMBITS(1) [],
        FLIP_VERT         OFFSET(7) NUMBITS(1) [],
    ]
];

#[repr(C)]
struct OAMSprite {
    y_pixel_coord: u8,
    tile_idx: u8,
    attribs: InMemoryRegister<u8, SpriteAttribs::Register>,
    x_pixel_coord: u8,
    current_x: u8,
    sprite_0: bool,
}

impl OAMSprite {
    pub fn from(data: &[u8], sprite_0: bool) -> Self {
        Self {
            y_pixel_coord: data[0],
            tile_idx: data[1],
            attribs: InMemoryRegister::new(data[2]),
            x_pixel_coord: data[3],
            current_x: data[3],
            sprite_0,
        }
    }
}

pub struct PPU {
    nametable_addr: u16,
    x_scroll: u8,
    y_scroll: u8,
    pub scanlines: usize,
    secondary_oam: Vec<OAMSprite>,
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
            secondary_oam: Vec::new(),
            dots: 21, // Simulates power-up delay
            generated_interrupt: false,
        }
    }

    pub fn step<T: FrameBuffer>(&mut self, fb: &mut T, bus: &mut Bus) -> bool {
        // At the start of each scanline, we have to check if a split x scroll occured...
        if self.dots == 0 {
            self.x_scroll = bus.ppu_get_registers().fine_x;
            // We only have to modify the coarse x scroll in the nametable addr
            self.nametable_addr.set_bit_range(4, 0, self.x_scroll / 8);
        }

        // Each step processes a single dot/pixel
        // Though in reality we don't render under the scanline is finished
        self.dots += 1;

        if self.dots == PPU::DOTS_PER_SCANLINE {
            // We just completed a scanline, render it
            // Don't bother drawing to the overdraw scanlines, they will never be seen anyway
            if self.scanlines <= 239 {
                self.draw_scanline(fb, bus);
                self.sprite_evaluation(self.scanlines + 1, bus);
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
            bus.ppu_get_registers_mut()
                .ppustatus
                .modify(PPUSTATUS::SPRITE0_HIT::CLEAR);
        }
        return false;
    }

    // TODO: Max 8 Sprites
    fn sprite_evaluation(&mut self, next_scanline: usize, bus: &mut Bus) {
        self.secondary_oam.clear();

        for (i, sprite_data) in bus.oam_ram.chunks(4).enumerate() {
            let y_coord = sprite_data[0] as usize;
            // TODO: IMPORTANT: Sprites are sometimes 16 pixels long!
            if (y_coord..y_coord + 8).contains(&next_scanline) {
                self.secondary_oam
                    .push(OAMSprite::from(sprite_data, i == 0));
            }
        }

        // Reverse the order, because we want to draw the earliest sprite last
        self.secondary_oam.reverse();
    }

    fn prepare_next_frame(&mut self, bus: &mut Bus) {
        self.scanlines = 0;
        // Update x_scroll and y_scroll
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
            let palette_num_bg = PPU::compute_bg_palette_num(attrib_table_val, coarse_x, coarse_y);

            // Get the chr tile data, a 16 byte chunk representing an individual 8x8 tile
            let tile = bus
                .ppu_get_pattern_table(true)
                .nth(pt_idx as usize)
                .unwrap();

            let palette_idx_bg = PPU::compute_bg_palette_idx(
                tile,
                fine_x_wrapped,
                fine_y_wrapped + pixel_space_y as u8,
            );
            let bg_color = bus
                .palette_memory
                .get_color_by_idx(palette_num_bg, palette_idx_bg)
                .unwrap();
            fb.plot_pixel(pixel_space_x, pixel_space_y, bg_color);

            // Handle sprites
            let bg_pixel_transparent = bus
                .palette_memory
                .is_entry_transparent(palette_num_bg, palette_idx_bg);
            let sprite_iter = self
                .secondary_oam
                .iter_mut()
                .filter(|sprite| sprite.current_x == pixel_space_x as u8);
            for sprite in sprite_iter {
                if sprite.current_x >= (sprite.x_pixel_coord + 8) {
                    continue; // No more drawing needed for this scanline
                }
                // Render a single pixel of a sprite
                let sprite_data = bus
                    .ppu_get_pattern_table(false)
                    .nth(sprite.tile_idx as usize)
                    .unwrap();
                let sprite_palette_idx = PPU::compute_palette_idx(
                    sprite_data,
                    pixel_space_x as u8 - sprite.x_pixel_coord,
                    pixel_space_y as u8 - sprite.y_pixel_coord,
                    sprite.attribs.is_set(SpriteAttribs::FLIP_HORZ),
                    sprite.attribs.is_set(SpriteAttribs::FLIP_VERT),
                );
                // if the sprite pixel isn't transparent...
                if sprite_palette_idx != 0 {
                    let sprite_palette_num: u8 = sprite.attribs.read(SpriteAttribs::PALETTE) + 4;
                    let sprite_color = bus
                        .palette_memory
                        .get_color_by_idx(sprite_palette_num, sprite_palette_idx)
                        .unwrap();

                    // Is this a sprite zero hit?
                    if sprite.sprite_0 && !bg_pixel_transparent {
                        bus.ppu_get_registers_mut()
                            .ppustatus
                            .modify(PPUSTATUS::SPRITE0_HIT::SET);
                    }

                    // Is the sprite pixel behind a transparent background pixel?
                    if sprite.attribs.is_set(SpriteAttribs::PRIORITY) && !bg_pixel_transparent {
                        // If this sprite pixel is meant to be drawn in the background,
                        // we must re-write the background pixel color into here
                        // We have to REWRITE the background color because the pixel color may have
                        // been adjusted by a previous sprite overlapping this sprite
                        fb.plot_pixel(pixel_space_x, pixel_space_y, bg_color);
                    } else {
                        fb.plot_pixel(pixel_space_x, pixel_space_y, sprite_color);
                    }
                }
                sprite.current_x += 1;
            }

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

    pub fn compute_palette_idx(
        tile_data: &[u8],
        x_coord: u8,
        y_coord: u8,
        flip_x: bool,
        flip_y: bool,
    ) -> u8 {
        // Read the palette idx data from both bitplanes in the tile
        let bit_idx = if flip_x {
            x_coord
        } else {
            7 - (x_coord) // Flip the bit index so we go from left to right over the bits
                          // The pattern table technically stores the sprites flipped horizontally by default
        };
        let y_tile_idx = if flip_y {
            7 - (y_coord % 8)
        } else {
            y_coord % 8
        };
        let low_bit = u8::from(tile_data[y_tile_idx as usize].bit(bit_idx as usize));
        let high_bit = u8::from(tile_data[(y_tile_idx + 8) as usize].bit(bit_idx as usize));
        low_bit + (high_bit << 1)
    }

    pub fn compute_bg_palette_idx(tile_data: &[u8], x_coord: u8, y_coord: u8) -> u8 {
        PPU::compute_palette_idx(tile_data, x_coord, y_coord, false, false)
    }

    pub fn compute_bg_palette_num(attrib_value: u8, coarse_x: u8, coarse_y: u8) -> u8 {
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
