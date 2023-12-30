pub struct NES;

impl eframe::App for NES {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
    }
}
