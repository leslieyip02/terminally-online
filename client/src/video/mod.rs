use crate::video::{
    encoding::{NalType, get_prefix_code},
    error::Error,
};

mod debug;
pub mod encoding;
pub mod error;
pub mod webcam;
pub struct VideoPanel {
    h264_decoder: openh264::decoder::Decoder,
    sps: Option<Vec<u8>>,
    pps: Option<Vec<u8>>,
    frame_buffer: Vec<u8>,
    rgb_buffer: Vec<u8>,
}

impl VideoPanel {
    pub fn new() -> Result<Self, Error> {
        let h264_decoder =
            openh264::decoder::Decoder::new().map_err(|e| Error::OpenH264 { error: e })?;

        Ok(Self {
            h264_decoder,
            sps: None,
            pps: None,
            frame_buffer: Vec::new(),
            rgb_buffer: Vec::new(),
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

        match self.save_frame() {
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
}
