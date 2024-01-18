use eframe::epaint::Color32;

use super::palette::lookup_palette_color;

pub struct PaletteMemory {
    memory: [u8; 32],
}

impl PaletteMemory {
    pub fn new() -> Self {
        Self { memory: [0u8; 32] }
    }

    pub fn set_entry(&mut self, addr: usize, val: u8) {
        match addr {
            0x3F00..=0x3F0F => self.memory[addr % 0x3F00] = val,
            0x3F10 => self.memory[0] = val, // Mirror of background
            0x3F11..=0x3F13 => self.memory[addr % 0x3F00] = val,
            0x3F14 => self.memory[4] = val, // Mirror of bkgrnd palette 0 transparent
            0x3F15..=0x3F17 => self.memory[addr % 0x3F00] = val,
            0x3F18 => self.memory[8] = val, // Mirror of bkgrnd palette 1 transparent
            0x3F19..=0x3F1B => self.memory[addr % 0x3F00] = val,
            0x3F1C => self.memory[0xC] = val, // Mirror of bkgrnd palette 2 transparent
            0x3F1D..=0x3F1F => self.memory[addr % 0x3F00] = val,
            _ => panic!("Invalid palette memory address!"),
        }
    }

    pub fn get_entry(&self, mut addr: usize) -> u8 {
        // Mirror transparent colors into the universal background color
        if addr % 4 == 0 {
            addr = 0x3F00;
        }

        match addr {
            0x3F00..=0x3F1F => self.memory[addr % 0x3F00],
            _ => panic!("Invalid palette memory address!"),
        }
    }

    pub fn is_entry_transparent(&self, palette_num: u8, idx: u8) -> bool {
        let palette_idx = (palette_num * 4) + idx;
        palette_idx % 4 == 0
    }

    pub fn get_color_by_idx(&self, palette_num: u8, idx: u8) -> Result<Color32, &'static str> {
        let addr = 0x3F00 + (palette_num as usize * 4) + idx as usize;
        let color_idx = self.get_entry(addr);
        lookup_palette_color(color_idx)
    }
}
