use std::io::{Error, ErrorKind};

use self::{bus::Bus, cpu::CPU, ppu::PPU, ui::UI};

mod bus;
mod cartridge;
mod cpu;
mod ines;
mod instruction;
mod mappers;
mod ppu;
mod ui;
mod util;

pub struct NES {
    cpu: CPU,
    ppu: PPU,
    bus: Bus,
    ui: UI,
    halt: bool,
}

impl NES {
    pub fn new(rom_path: &str) -> Result<Self, Error> {
        let bus = Bus::new(rom_path)?;
        let cpu = CPU::new(&bus).map_err(|_| Error::from(ErrorKind::AddrNotAvailable))?;
        Ok(Self {
            cpu,
            ppu: PPU::new(),
            bus,
            ui: UI::new(),
            halt: false,
        })
    }
}

impl eframe::App for NES {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        if !self.halt {
            // Since we don't have a PPU generating frames yet
            // we can just fake roughly how many cycles should be executed per frame
            for _ in 0..29781 {
                if let Err(error) = self.cpu.step(&mut self.bus) {
                    self.halt = true;
                    log::error!("Emulation failed with error: {}", error);
                    break;
                }
            }
        }

        self.ui.render(ctx, &mut self.bus);
        ctx.request_repaint();
    }
}
