use std::io::Error;

use self::{bus::BusImpl, cpu::CPU, ui::UI};

mod bus;
mod cartridge;
mod cpu;
mod ines;
mod instruction;
mod mappers;
mod ui;

pub struct NES {
    cpu: CPU,
    bus: BusImpl,
    ui: UI,
}

impl NES {
    pub fn new(rom_path: &str) -> Result<Self, Error> {
        Ok(Self {
            cpu: CPU::default(),
            bus: BusImpl::new(rom_path)?,
            ui: UI::new(),
        })
    }

    pub fn reset(&mut self, rom_path: &str) {
        self.cpu.reset();
    }
}

impl eframe::App for NES {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.cpu.step(&mut self.bus);
        self.ui.render(ctx, &mut self.bus);
        ctx.request_repaint();
    }
}
