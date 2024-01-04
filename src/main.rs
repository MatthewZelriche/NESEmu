use eframe::egui::ViewportBuilder;
use nes::NES;

mod nes;

fn main() {
    // TODO: Dehardcode rom path
    let path = "res/nestest.nes";
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
