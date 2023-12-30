use std::io::{Error, ErrorKind};

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
        let bus = BusImpl::new(rom_path)?;
        let cpu = CPU::new(&bus).map_err(|_| Error::from(ErrorKind::AddrNotAvailable))?;
        Ok(Self {
            cpu,
            bus,
            ui: UI::new(),
        })
    }
}

impl eframe::App for NES {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        self.cpu.step(&mut self.bus);
        self.ui.render(ctx, &mut self.bus);
        ctx.request_repaint();
    }
}
