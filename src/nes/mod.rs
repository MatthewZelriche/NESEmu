use self::{bus::BusImpl, cpu::CPU};

mod bus;
mod cpu;

pub struct NES {
    cpu: CPU,
    bus: BusImpl,
}

impl NES {
    pub fn new() -> Self {
        NES::default()
    }
}

impl Default for NES {
    fn default() -> Self {
        Self {
            cpu: Default::default(),
            bus: Default::default(),
        }
    }
}

impl eframe::App for NES {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
    }
}
