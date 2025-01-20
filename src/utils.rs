use egui::ahash::{HashSet, HashSetExt};
use egui::ColorImage;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::FrameFormat;
use nokhwa::Buffer;

pub fn remove_duplicates_by<T, F, K>(items: Vec<T>, key_selector: F) -> Vec<T>
where
    T: Clone,
    F: Fn(&T) -> K,
    K: Eq + std::hash::Hash,
{
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for item in items {
        let key = key_selector(&item);
        if seen.insert(key) {
            result.push(item);
        }
    }

    result
}

/// Convert buffer into image with format.
pub fn create_image_from_buffer(buffer: &Buffer, format: FrameFormat) -> Option<ColorImage> {
    let resolution = buffer.resolution();

    match format {
        FrameFormat::MJPEG | FrameFormat::RAWRGB => {
            if let Ok(rgb_buffer) = buffer.decode_image::<RgbFormat>() {
                Some(ColorImage::from_rgb(
                    [resolution.width() as usize, resolution.height() as usize],
                    rgb_buffer.as_raw(),
                ))
            } else {
                eprintln!("Error al decodificar MJPEG/RAWRGB.");
                None
            }
        }
        FrameFormat::YUYV => {
            let yuyv_buffer = buffer.buffer();
            let mut rgb_buffer = Vec::with_capacity(yuyv_buffer.len() * 2);

            for chunk in yuyv_buffer.chunks_exact(4) {
                if chunk.len() == 4 {
                    let (y0, u, y1, v) = (chunk[0], chunk[1], chunk[2], chunk[3]);
                    rgb_buffer.extend_from_slice(&yuv_to_rgb(y0, u, v));
                    rgb_buffer.extend_from_slice(&yuv_to_rgb(y1, u, v));
                }
            }

            Some(ColorImage::from_rgb(
                [resolution.width() as usize, resolution.height() as usize],
                &rgb_buffer,
            ))
        }
        FrameFormat::NV12 => {
            let (y_plane, uv_plane) = buffer
                .buffer()
                .split_at(resolution.width() as usize * resolution.height() as usize);

            let mut rgb_buffer = Vec::with_capacity(y_plane.len() * 3);
            for (i, &y) in y_plane.iter().enumerate() {
                let uv_index = (i / 2) * 2;
                let (u, v) = (uv_plane[uv_index], uv_plane[uv_index + 1]);
                rgb_buffer.extend_from_slice(&yuv_to_rgb(y, u, v));
            }

            Some(ColorImage::from_rgb(
                [resolution.width() as usize, resolution.height() as usize],
                &rgb_buffer,
            ))
        }
        FrameFormat::GRAY => {
            let rgb_buffer: Vec<u8> = buffer.buffer().iter().flat_map(|&g| [g, g, g]).collect();

            Some(ColorImage::from_rgb(
                [resolution.width() as usize, resolution.height() as usize],
                &rgb_buffer,
            ))
        }
    }
}

fn yuv_to_rgb(y: u8, u: u8, v: u8) -> [u8; 3] {
    let y = y as f32;
    let u = u as f32 - 128.0;
    let v = v as f32 - 128.0;

    let r = (y + 1.402 * v).clamp(0.0, 255.0) as u8;
    let g = (y - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
    let b = (y + 1.772 * u).clamp(0.0, 255.0) as u8;

    [r, g, b]
}
