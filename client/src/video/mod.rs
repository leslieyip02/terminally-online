use std::io::Write;

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, PrintStyledContent, Stylize},
};
use openh264::formats::YUVSource;

use crate::{
    layout::Drawable, video::{
        encoding::{get_prefix_code, NalType},
        error::Error,
        interpolater::BilinearInterpolater,
    }
};

pub mod encoding;
pub mod error;
pub mod webcam;

mod debug;
mod interpolater;

const UPPER_HALF_BLOCK: char = '▀';

pub struct VideoPanel {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    h264_decoder: openh264::decoder::Decoder,
    sps: Option<Vec<u8>>,
    pps: Option<Vec<u8>>,
    frame_buffer: Vec<u8>,
    rgb_buffer: Vec<u8>,
    bilinear_interpolater: BilinearInterpolater,
}

impl VideoPanel {
    const PADDING: u16 = 1;

    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Result<Self, Error> {
        let h264_decoder =
            openh264::decoder::Decoder::new().map_err(|e| Error::OpenH264 { error: e })?;
        let bilinear_interpolater =
            BilinearInterpolater::new(width - 2 * (Self::PADDING + 1), (height - 2) * 2);

        Ok(Self {
            x: x,
            y: y,
            width: width,
            height: height,
            h264_decoder: h264_decoder,
            sps: None,
            pps: None,
            frame_buffer: Vec::new(),
            rgb_buffer: Vec::new(),
            bilinear_interpolater: bilinear_interpolater,
        })
    }

    pub fn receive_stream(&mut self, stream: &Vec<u8>) -> Result<(), Error> {
        let mut contains_idr = false;
        for nal_unit in openh264::nal_units(&stream) {
            let nal_type = get_prefix_code(nal_unit)?;
            match nal_type {
                NalType::SPS => self.sps = Some(nal_unit.to_vec()),
                NalType::PPS => self.pps = Some(nal_unit.to_vec()),
                NalType::IDR => contains_idr = true,
                _ => {}
            }
        }

        if contains_idr {
            self.init_frame_buffer();
        }
        self.frame_buffer.extend_from_slice(&stream);

        match self.decode_frame() {
            Ok(_) => self.frame_buffer.clear(),
            Err(e) => {
                self.frame_buffer.clear();
                return Err(e);
            }
        }

        Ok(())
    }

    fn init_frame_buffer(&mut self) {
        self.frame_buffer.clear();
        if let (Some(sps), Some(pps)) = (&self.sps, &self.pps) {
            self.frame_buffer.extend_from_slice(sps);
            self.frame_buffer.extend_from_slice(pps);
        }
    }

    fn decode_frame(&mut self) -> Result<(), Error> {
        let decoded = self
            .h264_decoder
            .decode(&self.frame_buffer)
            .map_err(|e| Error::OpenH264 { error: e })?
            .ok_or_else(|| Error::Decoding)?;

        let (width, height) = decoded.dimensions();
        let need_resize = self.rgb_buffer.len() != width * height * 3;
        if need_resize {
            self.rgb_buffer.resize(width * height * 3, 0);
        }
        decoded.write_rgb8(&mut self.rgb_buffer);

        if need_resize {
            self.bilinear_interpolater.update_weights(width, height);
        }
        self.bilinear_interpolater
            .update_grayscale_buffer(&self.rgb_buffer);

        Ok(())
    }
}

impl Drawable for VideoPanel {
    fn draw(&self, stdout: &mut std::io::Stdout) -> Result<(), std::io::Error> {
        self.bilinear_interpolater
            .grouped_rows()
            .enumerate()
            .try_for_each(|(y, rows)| -> Result<(), std::io::Error> {
                let width = rows.len() / 2;
                for x in 0..width {
                    let tile = UPPER_HALF_BLOCK
                        .with(Color::AnsiValue(normalize(rows[x])))
                        .on(Color::AnsiValue(normalize(rows[width + x])));
                    stdout
                        .queue(MoveTo(
                            x as u16 + self.x + Self::PADDING + 1,
                            y as u16 + self.y + 1,
                        ))?
                        .queue(PrintStyledContent(tile))?;
                }

                Ok(())
            })?;

        stdout.flush()
    }

    fn x(&self) -> u16 {
        self.x
    }

    fn y(&self) -> u16 {
        self.y
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }
}

fn normalize(value: u8) -> u8 {
    232 + ((value as f32) / 256.0 * 24.0) as u8
}
