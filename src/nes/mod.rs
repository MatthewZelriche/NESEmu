use std::io::Error;

use self::{bus::BusImpl, cpu::CPU};

mod bus;
mod cartridge;
mod cpu;
mod ines;
mod mappers;

pub struct NES {
    cpu: CPU,
    bus: BusImpl,
}

impl NES {
    pub fn new(rom_path: &str) -> Result<Self, Error> {
        Ok(Self {
            cpu: CPU::default(),
            bus: BusImpl::new(rom_path)?,
        })
    }

    pub fn reset(&mut self, rom_path: &str) {
        self.cpu.reset();
    }
}

impl eframe::App for NES {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
    }
}
