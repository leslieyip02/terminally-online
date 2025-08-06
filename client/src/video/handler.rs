use openh264::formats::YUVSource;

use crate::video::{
    encoding::{NalType, get_prefix_code},
    error::Error,
};

pub trait VideoHandler {
    fn receive_stream(&mut self, stream: &Vec<u8>) -> Result<(usize, usize), Error>;
    fn rgb_buffer(&self) -> &[u8];
}

pub struct PeerVideoHandler {
    h264_decoder: openh264::decoder::Decoder,
    sps: Option<Vec<u8>>,
    pps: Option<Vec<u8>>,
    frame_buffer: Vec<u8>,
    rgb_buffer: Vec<u8>,
}

impl PeerVideoHandler {
    pub fn new() -> Result<Self, Error> {
        let h264_decoder =
            openh264::decoder::Decoder::new().map_err(|e| Error::OpenH264 { error: e })?;

        Ok(Self {
            h264_decoder: h264_decoder,
            sps: None,
            pps: None,
            frame_buffer: Vec::new(),
            rgb_buffer: Vec::new(),
        })
    }

    fn init_frame_buffer(&mut self) {
        self.frame_buffer.clear();
        if let (Some(sps), Some(pps)) = (&self.sps, &self.pps) {
            self.frame_buffer.extend_from_slice(sps);
            self.frame_buffer.extend_from_slice(pps);
        }
    }

    fn decode_frame(&mut self) -> Result<(usize, usize), Error> {
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
        Ok((width, height))
    }
}

impl VideoHandler for PeerVideoHandler {
    fn receive_stream(&mut self, stream: &Vec<u8>) -> Result<(usize, usize), Error> {
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

        let decoded_result = self.decode_frame();
        self.frame_buffer.clear();

        decoded_result
    }

    fn rgb_buffer(&self) -> &[u8] {
        &self.rgb_buffer
    }
}
