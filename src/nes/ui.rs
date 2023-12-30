use eframe::egui::Context;
use egui_memory_editor::MemoryEditor;

use super::bus::Bus;

pub struct UI {
    mem_editor: MemoryEditor,
    mem_editor_open: bool,
}

impl UI {
    pub fn new() -> Self {
        Self {
            mem_editor: MemoryEditor::new()
                .with_address_range("All", 0..0xFFFF)
                .with_address_range("RAM", 0xFF00..0x0800)
                .with_window_title("Memory"),
            mem_editor_open: true,
        }
    }

    pub fn render<T: Bus>(&mut self, ctx: &Context, bus: &mut T) {
        self.mem_editor.window_ui(
            ctx,
            &mut self.mem_editor_open,
            bus,
            |bus, address| bus.read_byte(address).ok(),
            |bus, address, val| {
                // Discard error result, this memory editor doesn't need it
                let _ = bus.write_byte(address, val);
            },
        );
    }
}
