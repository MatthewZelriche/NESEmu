use std::io::{Error, ErrorKind};

use bitfield::BitMut;
use eframe::{
    egui::{Image, Key, Window},
    CreationContext,
};

use self::{bus::Bus, controller::InputEvent, cpu::CPU, ppu::PPU, screen::Screen, ui::UI};

mod bus;
mod cartridge;
mod controller;
mod cpu;
mod ines;
mod instruction;
mod mappers;
mod palette;
mod palette_memory;
mod ppu;
mod ppu_registers;
mod screen;
mod ui;
mod util;

pub struct NES {
    cpu: CPU,
    ppu: PPU,
    bus: Bus,
    ui: UI,
    halt: bool,
    screen: Screen,
    pending_interrupt: bool,
}

impl NES {
    pub fn new(rom_path: &str, cc: &CreationContext) -> Result<Self, Error> {
        let mut bus = Bus::new(rom_path)?;
        let cpu = CPU::new(&mut bus).map_err(|_| Error::from(ErrorKind::AddrNotAvailable))?;
        Ok(Self {
            cpu,
            ppu: PPU::new(),
            bus,
            ui: UI::new(),
            halt: false,
            screen: Screen::new(cc.egui_ctx.clone()),
            pending_interrupt: false,
        })
    }

    // TODO: Dehardcode keys
    pub fn handle_window_input(&self, ctx: &eframe::egui::Context) -> InputEvent {
        let mut event = InputEvent { input_state: 0 };
        ctx.input(|info| {
            event
                .input_state
                .set_bit(InputEvent::RIGHT as usize, info.key_down(Key::ArrowRight));
            event
                .input_state
                .set_bit(InputEvent::LEFT as usize, info.key_down(Key::ArrowLeft));
            event
                .input_state
                .set_bit(InputEvent::DOWN as usize, info.key_down(Key::ArrowDown));
            event
                .input_state
                .set_bit(InputEvent::UP as usize, info.key_down(Key::ArrowUp));
            event
                .input_state
                .set_bit(InputEvent::START as usize, info.key_down(Key::Enter));
            event
                .input_state
                .set_bit(InputEvent::SELECT as usize, info.key_down(Key::Backspace));
            event
                .input_state
                .set_bit(InputEvent::B as usize, info.key_down(Key::Z));
            event
                .input_state
                .set_bit(InputEvent::A as usize, info.key_down(Key::X));
        });
        event
    }
}

impl eframe::App for NES {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        self.bus
            .controller
            .set_state_from_window(self.handle_window_input(ctx));

        let mut did_finish_frame = false;
        if !self.halt {
            // Since we don't have a PPU generating frames yet
            // we can just fake roughly how many cycles should be executed per frame
            loop {
                self.pending_interrupt = self.ppu.generated_interrupt();
                match self.cpu.step(&mut self.bus, &mut self.pending_interrupt) {
                    Ok(_) => {
                        // 3 cycles per CPU cycle
                        for _ in 0..3 {
                            // Detect when the GPU finished all of its scanlines and
                            // looped back over to scanline 0
                            let res = self.ppu.step(&mut self.bus);
                            if !did_finish_frame && res {
                                did_finish_frame = res;
                            }
                        }
                        if did_finish_frame {
                            break;
                        }
                    }
                    Err(error) => {
                        self.halt = true;
                        log::error!("Emulation failed with error: {}", error);
                        break;
                    }
                }
            }
        }

        if did_finish_frame {
            // Render the frame to the screen
            self.ppu
                .draw_to_framebuffer(&mut self.screen, &mut self.bus);
            self.screen.update_texture();
        }

        self.ui.render(ctx, &mut self.bus);
        Window::new("Game").show(ctx, |ui| ui.add(Image::new(&self.screen.texture)));

        ctx.request_repaint();
    }
}
