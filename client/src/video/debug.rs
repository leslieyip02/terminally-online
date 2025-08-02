use image::{ColorType, save_buffer_with_format};
use openh264::formats::YUVSource;
use tracing::info;

use crate::video::{VideoPanel, error::Error};

impl VideoPanel {
    pub(crate) fn save_frame(&mut self) -> Result<(), Error> {
        if self.frame_buffer.is_empty() {
            return Ok(());
        }

        let decoded = self
            .h264_decoder
            .decode(&self.frame_buffer)
            .map_err(|e| Error::OpenH264 { error: e })?
            .ok_or_else(|| Error::Decoding)?;

        let (width, height) = decoded.dimensions();
        info!("decoded frame: {} x {}", width, height);

        // Resize buffer if needed
        if self.rgb_buffer.len() != width * height * 3 {
            self.rgb_buffer.resize(width * height * 3, 0);
        }

        decoded.write_rgb8(&mut self.rgb_buffer);

        let path = "debug.png";
        save_buffer_with_format(
            path,
            &self.rgb_buffer,
            width as u32,
            height as u32,
            ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .map_err(|e| Error::DebugImage {
            message: e.to_string(),
        })?;

        info!("decoded frame saved as {}", path);
        Ok(())
    }
}
