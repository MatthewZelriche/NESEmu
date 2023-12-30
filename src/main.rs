use nes::NES;

mod nes;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("NESEmu", native_options, Box::new(|_| Box::new(NES {})))
        .expect("Failed to start eframe");
}
