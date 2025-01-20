use egui::{Button, ColorImage, Label, TextureHandle, TextureOptions};
use egui_cameras::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "egui_cameras simple",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(MainApp::new(cc)))),
    )
}

pub struct MainApp {
    texture: TextureHandle,
    camera_manager: CameraManager,
}

impl MainApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let texture = cc.egui_ctx.load_texture(
            "test_camera",
            ColorImage::example(),
            TextureOptions::NEAREST,
        );
        let mut camera_manager = CameraManager::new(texture.clone());
        let camera = camera_manager.available_cameras()[1].clone();
        camera_manager.select_camera(Some(camera));
        camera_manager.start_capture(nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate);

        Self {
            texture,
            camera_manager,
        }
    }
}

impl eframe::App for MainApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.camera_manager.is_running()
                && self.camera_manager.get_frame(TextureOptions::NEAREST)
            {
                ui.image(&self.texture);
                ctx.request_repaint();
            } else {
                ui.label("No input camera");
            }
        });
    }
}
