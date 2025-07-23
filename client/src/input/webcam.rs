use nokhwa::{
    Camera, NokhwaError,
    pixel_format::{RgbFormat, YuyvFormat},
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution},
};

use crate::input::VideoFeed;

pub struct Webcam {
    camera: Camera,
}

impl Webcam {
    pub fn new() -> Result<Self, NokhwaError> {
        let index = CameraIndex::Index(0);
        let format = RequestedFormat::new::<YuyvFormat>(RequestedFormatType::None);
        let mut camera = Camera::new(index, format).unwrap();
        camera.open_stream().unwrap();

        Ok(Self { camera: camera })
    }
}

impl VideoFeed for Webcam {
    fn write_next_frame(&mut self, buffer: &mut [u8]) -> Result<(), Box<dyn std::error::Error>> {
        let frame = self.camera.frame()?;
        Ok(frame.decode_image_to_buffer::<RgbFormat>(buffer)?)
    }

    fn resolution(&self) -> Resolution {
        self.camera.resolution()
    }
}
