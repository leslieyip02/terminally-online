use image::{ColorType, save_buffer_with_format};
use openh264::{decoder::Decoder, formats::YUVSource};
use tracing::info;

use crate::video::error::Error;

pub mod encoding;
pub mod error;
pub mod webcam;
pub struct VideoPanel {
    h264_decoder: Decoder,
    rgb_buffer: Vec<u8>,
    sps: Option<Vec<u8>>,
    pps: Option<Vec<u8>>,
    frame_buffer: Vec<u8>,
}

impl VideoPanel {
    pub fn new() -> Result<Self, Error> {
        let h264_decoder = Decoder::new().map_err(|e| Error::OpenH264 { error: e })?;
        Ok(Self {
            h264_decoder,
            rgb_buffer: Vec::new(),
            sps: None,
            pps: None,
            frame_buffer: Vec::new(),
        })
    }

    pub fn receive_frame(&mut self, frame: &Vec<u8>) -> Result<(), Error> {
        let mut contains_idr = false;

        // TODO: fix this
        for nal_unit in openh264::nal_units(&frame) {
            let nal_type = nal_unit[0] & 0x1F;

            match nal_type {
                7 => {
                    self.sps = Some(nal_unit.to_vec());
                    info!("stored sps ({} bytes)", nal_unit.len());
                }
                8 => {
                    self.pps = Some(nal_unit.to_vec());
                    info!("stored pps ({} bytes)", nal_unit.len());
                }
                5 => {
                    contains_idr = true;
                }
                _ => {}
            }
        }

        // Add SPS/PPS if needed
        if contains_idr {
            self.frame_buffer.clear();

            if let (Some(sps), Some(pps)) = (&self.sps, &self.pps) {
                self.frame_buffer.extend_from_slice(&[0, 0, 0, 1]);
                self.frame_buffer.extend_from_slice(sps);
                self.frame_buffer.extend_from_slice(&[0, 0, 0, 1]);
                self.frame_buffer.extend_from_slice(pps);
            } else {
                info!("missing SPS/PPS — decoder may fail");
            }
        }

        self.frame_buffer.extend_from_slice(&frame);
        match self.save_frame() {
            Ok(_) => {
                self.frame_buffer.clear();
            }
            Err(e) => {
                self.frame_buffer.clear();
                return Err(e);
            }
        }

        Ok(())
    }

    fn save_frame(&mut self) -> Result<(), Error> {
        if self.frame_buffer.is_empty() {
            return Ok(());
        }

        info!("decoding frame of {} bytes", self.frame_buffer.len());

        let decoded = self
            .h264_decoder
            .decode(&self.frame_buffer)
            .map_err(|e| {
                info!("decoder error: {:?}", e);
                Error::OpenH264 { error: e }
            })?
            .ok_or_else(|| {
                info!("decoder returned None - frame may be incomplete");
                Error::Decoding
            })?;

        let (width, height) = decoded.dimensions();
        info!("Decoded frame: {}x{}", width, height);

        // Resize buffer if needed
        if self.rgb_buffer.len() != width * height * 3 {
            self.rgb_buffer.resize(width * height * 3, 0);
        }

        decoded.write_rgb8(&mut self.rgb_buffer);

        save_buffer_with_format(
            "tmp.png",
            &self.rgb_buffer,
            width as u32,
            height as u32,
            ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .map_err(|e| {
            info!("image save error: {:?}", e);
            Error::Decoding
        })?;

        info!("decoded frame saved as tmp.png");
        Ok(())
    }
}
