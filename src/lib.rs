use egui::mutex::Mutex;
use egui::{TextureHandle, TextureOptions};
use nokhwa::pixel_format::RgbAFormat;
use nokhwa::utils::{ApiBackend, CameraInfo, RequestedFormat, RequestedFormatType};
use nokhwa::{nokhwa_initialize, query, CallbackCamera};

use utils::remove_duplicates_by;

use self::utils::create_image_from_buffer;

mod utils;

pub struct CameraManager {
    running: bool,
    texture: Mutex<TextureHandle>,
    selected_camera: Option<CallbackCamera>,
    selected_camera_info: Option<CameraInfo>,
    available_cameras: Vec<CameraInfo>,
}

impl CameraManager {
    pub fn new(texture: TextureHandle) -> Self {
        nokhwa_initialize(|_| {});

        let mut available_cameras = query(ApiBackend::Auto).unwrap_or_default();
        available_cameras = remove_duplicates_by(available_cameras, |cam| {
            format!("{:?}->{}", cam.index(), cam.human_name())
        });

        Self {
            running: false,
            available_cameras,
            selected_camera: None,
            selected_camera_info: None,
            texture: Mutex::new(texture),
        }
    }

    pub fn select_camera(&mut self, camera: Option<CameraInfo>) {
        self.selected_camera_info = camera;
    }

    pub fn available_cameras(&self) -> &[CameraInfo] {
        &self.available_cameras
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn stop(&mut self) {
        if let Some(cam) = self.selected_camera.as_mut() {
            cam.stop_stream().unwrap();
        }
        self.selected_camera = None;
        self.selected_camera_info = None;
        self.running = false;
    }

    pub fn start_capture(&mut self, format: RequestedFormatType) {
        if self.selected_camera.is_none()
            || self
                .selected_camera
                .as_ref()
                .zip(self.selected_camera_info.as_ref())
                .is_some_and(|(camera, selected)| camera.info() != selected)
        {
            let format_request = RequestedFormat::new::<RgbAFormat>(format);
            if let Some(cam_info) = self.selected_camera_info.as_ref() {
                let texture = self.texture.clone();
                let mut selected_camera =
                    CallbackCamera::new(cam_info.index().clone(), format_request, move |buffer| {
                        let format = buffer.source_frame_format();
                        if let Some(image) = create_image_from_buffer(&buffer, format) {
                            texture.lock().set(image, TextureOptions::NEAREST);
                        }
                    })
                    .ok();

                if let Some(cam) = selected_camera.as_mut() {
                    cam.open_stream().unwrap_or_else(|e| {
                        eprintln!("Error open camera: {:?}", e);
                    });
                }
                self.selected_camera = selected_camera;
                self.running = true;
            }
        }
    }

    pub fn get_frame(&mut self, options: TextureOptions) -> bool {
        let Some(camera) = self.selected_camera.as_mut() else {
            return false;
        };
        let Ok(buffer) = camera.last_frame() else {
            return false;
        };
        let format = buffer.source_frame_format();
        let Some(image) = create_image_from_buffer(&buffer, format) else {
            return false;
        };

        self.texture.lock().set(image, options);

        true
    }
}
