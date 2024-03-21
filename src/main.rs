use eframe::egui::ViewportBuilder;
use nes::NES;
use std::env;

mod nes;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Missing rom path! Usage: cargo run <path/to/rom>");
        std::process::exit(-1);
    }

    let path = args[1].clone();
    let mut native_options = eframe::NativeOptions::default();
    native_options.vsync = false;
    native_options.viewport = ViewportBuilder::default().with_inner_size([1024.0, 768.0]);
    eframe::run_native(
        "NESEmu",
        native_options,
        Box::new(|cc| {
            Box::new(match NES::new(path, cc) {
                Ok(nes) => nes,
                Err(error) => panic!("failed to initialize NES with error: {}", error),
            })
        }),
    )
    .expect("Failed to start eframe");
}
