use eframe::egui::{Context, Window};
use egui_memory_editor::MemoryEditor;

use super::bus::Bus;

pub struct UI {
    mem_editor: MemoryEditor,
    mem_editor_open: bool,
}

impl UI {
    pub fn new() -> Self {
        egui_logger::init().unwrap();
        Self {
            mem_editor: MemoryEditor::new()
                .with_address_range("All", 0..0xFFFF)
                .with_address_range("RAM", 0..0x0800)
                .with_window_title("Memory"),
            mem_editor_open: true,
        }
    }

    pub fn render(&mut self, ctx: &Context, bus: &mut Bus) {
        self.mem_editor.window_ui(
            ctx,
            &mut self.mem_editor_open,
            bus,
            |bus, address| bus.cpu_read_byte(address).ok(),
            |bus, address, val| {
                // Discard error result, this memory editor doesn't need it
                let _ = bus.cpu_write_byte(address, val);
            },
        );
        Window::new("Log").show(ctx, |ui| {
            ui.style_mut().override_text_style = Some(eframe::egui::TextStyle::Monospace);
            // draws the logger ui.
            egui_logger::logger_ui(ui);
        });
    }
}
