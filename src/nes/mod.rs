use std::{
    io::{Error, ErrorKind},
    time::{Duration, Instant},
};

use bitfield::BitMut;
use eframe::{
    egui::{Image, Key, Vec2, Window},
    CreationContext,
};

use self::{bus::Bus, controller::InputEvent, cpu::CPU, ppu::PPU, screen::Screen, ui::UI};

mod bus;
mod controller;
mod cpu;
mod mappers;
mod ppu;
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
    frame_start: Instant,
    dma_read_cycle: bool,
}

impl NES {
    const FRAME_TIME: f64 = 1.0 / 60.098814;
    pub fn new(rom_path: &str, cc: &CreationContext) -> Result<Self, Error> {
        let mut bus = Bus::new(rom_path)?;
        let cpu = CPU::new(&mut bus).map_err(|_| Error::from(ErrorKind::AddrNotAvailable))?;
        Ok(Self {
            cpu,
            ppu: PPU::new(),
            bus,
            ui: UI::new(),
            halt: true,
            screen: Screen::new(cc.egui_ctx.clone()),
            pending_interrupt: false,
            frame_start: Instant::now(),
            dma_read_cycle: true,
        })
    }

    // TODO: Dehardcode keys
    pub fn handle_window_input(&mut self, ctx: &eframe::egui::Context) -> InputEvent {
        let mut event = InputEvent { input_state: 0 };
        ctx.input(|info| {
            if info.key_pressed(Key::P) {
                self.halt = !self.halt;
            }

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
        let input_event = self.handle_window_input(ctx);
        self.bus.controller.set_state_from_window(input_event);

        let mut did_finish_frame = false;
        if !self.halt {
            // Since we don't have a PPU generating frames yet
            // we can just fake roughly how many cycles should be executed per frame
            loop {
                self.pending_interrupt = self.ppu.generated_interrupt();

                let cycles: u16 = if self.dma_read_cycle && self.bus.pending_dma() {
                    self.bus.process_dma();
                    513 // Number of cycles it takes for a DMA transfer
                } else {
                    match self.cpu.step(&mut self.bus, &mut self.pending_interrupt) {
                        Ok(cycles) => cycles as u16,
                        Err(error) => {
                            self.halt = true;
                            log::error!("Emulation failed with error: {}", error);
                            break;
                        }
                    }
                };

                // 3 cycles per CPU cycle
                for _ in 0..(3 * cycles) {
                    // Detect when the GPU finished all of its scanlines and
                    // looped back over to scanline 0
                    let res = self.ppu.step(&mut self.screen, &mut self.bus);
                    if !did_finish_frame && res {
                        did_finish_frame = res;
                    }
                }
                if did_finish_frame {
                    break;
                }

                self.dma_read_cycle = !self.dma_read_cycle;
            }
        }

        if did_finish_frame {
            // Present the frame to the screen
            self.screen.update_texture();
        }

        self.ui.render(ctx, &mut self.bus);
        Window::new("Game").show(ctx, |ui| {
            ui.add(Image::new(&self.screen.texture).fit_to_exact_size(Vec2::new(512.0, 480.0)))
        });

        ctx.request_repaint();

        let ft = Duration::from_secs_f64(NES::FRAME_TIME);
        let duration = Instant::now() - self.frame_start;
        if ft > duration {
            spin_sleep::sleep(ft - duration);
        }

        self.frame_start = Instant::now();
    }
}
