use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution},
};
use openh264::{
    encoder::Encoder,
    formats::{RgbSliceU8, YUVBuffer},
};

use crate::input::VideoFeed;

pub struct Webcam {
    camera: Camera,
    camera_dimensions: (usize, usize),
    rgb_buffer: Vec<u8>,
    yuv_buffer: YUVBuffer,
    h264_encoder: Encoder,
}

impl Webcam {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let index = CameraIndex::Index(0);
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
        let mut camera = Camera::new(index, format).unwrap();
        camera.open_stream().unwrap();

        let input_width = camera.resolution().width() as usize;
        let input_height = camera.resolution().height() as usize;
        let input_buffer_size = input_width * input_height * 3;
        let rgb_buffer = vec![0; input_buffer_size];

        let yuv_buffer = YUVBuffer::new(input_width, input_height);
        let h264_encoder = Encoder::new()?;

        Ok(Self {
            camera: camera,
            camera_dimensions: (input_width, input_height),
            h264_encoder: h264_encoder,
            yuv_buffer: yuv_buffer,
            rgb_buffer: rgb_buffer,
        })
    }

    fn encode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = self.h264_encoder.encode(&self.yuv_buffer)?;

        // TODO: send encoded stream

        Ok(())
    }
}

impl VideoFeed for Webcam {
    fn write_next_frame(&mut self, buffer: &mut [u8]) -> Result<(), Box<dyn std::error::Error>> {
        let frame = self.camera.frame()?;

        frame.decode_image_to_buffer::<RgbFormat>(&mut self.rgb_buffer)?;
        let slice = RgbSliceU8::new(&self.rgb_buffer, self.camera_dimensions);
        self.yuv_buffer.read_rgb8(slice);

        // TODO: refactor?
        buffer.copy_from_slice(&self.rgb_buffer);
        Ok(())
    }

    fn resolution(&self) -> Resolution {
        self.camera.resolution()
    }
}
