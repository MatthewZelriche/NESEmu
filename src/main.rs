use nes::NES;

mod nes;

fn main() {
    // TODO: Dehardcode rom path
    let path = "res/nestest.nes";
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "NESEmu",
        native_options,
        Box::new(|_| {
            Box::new(match NES::new(path) {
                Ok(nes) => nes,
                Err(error) => panic!("failed to initialize NES with error: {}", error),
            })
        }),
    )
    .expect("Failed to start eframe");
}
