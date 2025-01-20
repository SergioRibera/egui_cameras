use egui::{ColorImage, ComboBox, Image, TextureHandle, TextureOptions};
use egui_cameras::*;
use nokhwa::utils::{CameraInfo, FrameFormat, Resolution};

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
    cameras: Vec<CameraInfo>,
    selected_camera: CameraInfo,
    selected_res: Resolution,
    selected_framerate: u32,
    selected_format: FrameFormat,
    available_resolutions: Vec<(Resolution, Vec<u32>)>,
    available_formats: [FrameFormat; 5],
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
        let cameras = camera_manager.available_cameras();
        let selected_camera = cameras[0].clone();
        camera_manager.select_camera(Some(selected_camera.clone()));
        camera_manager.start_capture(nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate);

        // Inicializar valores predeterminados
        let available_resolutions = camera_manager.available_resolutions(FrameFormat::YUYV);
        let selected_res = available_resolutions
            .first()
            .map(|(res, _)| res.clone())
            .unwrap_or_default();
        let selected_framerate = available_resolutions
            .first()
            .map(|(_, rates)| rates[0])
            .unwrap_or_default();
        let available_formats = [
            FrameFormat::YUYV,
            FrameFormat::MJPEG,
            FrameFormat::RAWRGB,
            FrameFormat::GRAY,
            FrameFormat::NV12,
        ];
        let selected_format = available_formats[0].clone();

        Self {
            texture,
            cameras,
            camera_manager,
            selected_camera,
            selected_res,
            selected_framerate,
            selected_format,
            available_resolutions,
            available_formats,
        }
    }

    fn update_available_resolutions(&mut self) {
        self.available_resolutions = self
            .camera_manager
            .available_resolutions(self.selected_format.clone());
        self.available_resolutions
            .sort_by(|(res_a, _), (res_b, _)| {
                (res_a.width() * res_a.height()).cmp(&(res_b.width() * res_b.height()))
            });
        self.selected_res = self
            .available_resolutions
            .first()
            .map(|(res, _)| res.clone())
            .unwrap_or_default();
        self.selected_framerate = self
            .available_resolutions
            .first()
            .map(|(_, rates)| rates[0])
            .unwrap_or_default();
    }
}

impl eframe::App for MainApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                let w = ui.available_width() * 0.3;

                if self.camera_manager.get_frame(TextureOptions::LINEAR) {
                    ui.set_max_width(w);
                    ui.add(Image::new(&self.texture).max_width(w));
                    ctx.request_repaint();
                } else {
                    ui.label("No input camera");
                }

                ui.add_space(20.0);
                ui.horizontal_top(|ui| {
                    ui.label("Select Camera");
                    ui.add_space(20.0);
                    let selected = self.selected_camera.clone();
                    ComboBox::from_id_salt("Select Camera")
                        .truncate()
                        .selected_text(selected.human_name())
                        .show_ui(ui, |ui| {
                            let mut changed = None;
                            for c in &self.cameras {
                                if ui
                                    .selectable_value(
                                        &mut self.selected_camera,
                                        c.clone(),
                                        c.human_name(),
                                    )
                                    .changed()
                                {
                                    changed = Some(c.clone());
                                }
                            }

                            if changed.is_some() {
                                self.camera_manager.select_camera(changed);
                                self.camera_manager.start_capture(
                                    nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate,
                                );
                                self.update_available_resolutions();
                            }
                        });
                });

                ui.horizontal_top(|ui| {
                    ui.label("Select FrameFormat");
                    ui.add_space(20.0);
                    ComboBox::from_id_salt("Select FrameFormat")
                        .truncate()
                        .selected_text(format!("{:?}", self.selected_format))
                        .show_ui(ui, |ui| {
                            let mut changed = false;
                            for format in &self.available_formats {
                                if ui
                                    .selectable_value(
                                        &mut self.selected_format,
                                        format.clone(),
                                        format!("{:?}", format),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            }
                            if changed {
                                self.update_available_resolutions();
                            }
                        });
                });

                ui.horizontal_top(|ui| {
                    ui.label("Select Resolution");
                    ui.add_space(20.0);
                    ComboBox::from_id_salt("Select Resolution")
                        .truncate()
                        .selected_text(self.selected_res.to_string())
                        .show_ui(ui, |ui| {
                            for (res, _) in &self.available_resolutions {
                                if ui
                                    .selectable_value(
                                        &mut self.selected_res,
                                        res.clone(),
                                        res.to_string(),
                                    )
                                    .changed()
                                {
                                    let frame_rates = self
                                        .available_resolutions
                                        .iter()
                                        .find(|(r, _)| r == res)
                                        .map(|(_, rates)| rates.clone())
                                        .unwrap_or_default();
                                    self.selected_framerate = frame_rates[0];
                                    self.camera_manager.set_resolution(res.clone());
                                }
                            }
                        });
                });

                ui.horizontal_top(|ui| {
                    ui.label("Select FrameRate");
                    ui.add_space(20.0);
                    ComboBox::from_id_salt("Select FrameRate")
                        .truncate()
                        .selected_text(format!("{} FPS", self.selected_framerate))
                        .show_ui(ui, |ui| {
                            if let Some((_, rates)) = self
                                .available_resolutions
                                .iter()
                                .find(|(res, _)| res == &self.selected_res)
                            {
                                for rate in rates {
                                    if ui
                                        .selectable_value(
                                            &mut self.selected_framerate,
                                            *rate,
                                            format!("{} FPS", rate),
                                        )
                                        .changed()
                                    {
                                        self.camera_manager.set_framerate(*rate);
                                    }
                                }
                            }
                        });
                });
            });
        });
    }
}
