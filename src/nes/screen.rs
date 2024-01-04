use eframe::{
    egui::{Context, TextureOptions},
    epaint::{Color32, ColorImage, TextureHandle},
};

pub struct Screen {
    pub frame_buffer: ColorImage,
    pub texture: TextureHandle,
}

pub trait FrameBuffer {
    fn plot_pixel(x: usize, y: usize, color: Color32);
}

impl Screen {
    const HEIGHT: usize = 240;
    const WIDTH: usize = 256;
    pub fn new(ctx: Context) -> Self {
        let frame_buffer = ColorImage::new([Screen::WIDTH, Screen::HEIGHT], Color32::BLACK);
        let texture = ctx.load_texture("Screen", frame_buffer.clone(), TextureOptions::default());
        Self {
            frame_buffer,
            texture,
        }
    }

    pub fn update_texture(&mut self) {
        // Update the texture
        // This seems very inefficient to be cloning this every frame, but it doesn't
        // seem possible to extract the image data once ive handed it over to the GPU
        self.texture
            .set(self.frame_buffer.clone(), TextureOptions::default())
    }
}

impl FrameBuffer for Screen {
    fn plot_pixel(x: usize, y: usize, color: Color32) {
        todo!()
    }
}
