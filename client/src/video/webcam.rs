use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bytes::Bytes;
use futures::Stream;
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use openh264::formats::{RgbSliceU8, YUVBuffer};
use tokio::{
    sync::mpsc::{UnboundedReceiver, unbounded_channel},
    time::Instant,
};
use tracing::info;

use crate::video::encoding::get_prefix_code;

pub struct Webcam {
    receiver: Option<UnboundedReceiver<Bytes>>,
}

impl Webcam {
    const FRAME_DURATION: Duration = Duration::from_millis(50);

    pub fn new() -> Self {
        Self { receiver: None }
    }

    pub fn start(&mut self) {
        let (sender, receiver) = unbounded_channel();
        self.receiver = Some(receiver);

        std::thread::spawn(move || {
            info!("starting webcam");

            let index = CameraIndex::Index(0);
            let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
            let mut camera = match Camera::new(index, format) {
                Ok(camera) => camera,
                Err(e) => {
                    info!("unable to create camera: {e}");
                    return;
                }
            };

            if let Err(e) = camera.open_stream() {
                info!("failed to start camera stream: {e}");
                return;
            }

            let input_width = camera.resolution().width() as usize;
            let input_height = camera.resolution().height() as usize;
            let input_buffer_size = input_width * input_height * 3;
            let camera_dimensions = (input_width, input_height);

            let mut rgb_buffer = vec![0; input_buffer_size];
            let mut yuv_buffer = YUVBuffer::new(input_width, input_height);

            let mut h264_encoder = match openh264::encoder::Encoder::new() {
                Ok(h264_encoder) => h264_encoder,
                Err(e) => {
                    info!("unable to create h264 encoder: {e}");
                    return;
                }
            };

            let mut h264_encoded_buffer = Vec::new();

            let mut frame_count = 0;
            let keyframe_interval = 30;

            loop {
                let frame_start = Instant::now();
                let frame = match camera.frame() {
                    Ok(frame) => frame,
                    Err(e) => {
                        info!("failed to get camera frame: {e}");
                        continue;
                    }
                };

                if let Err(e) = frame.decode_image_to_buffer::<RgbFormat>(&mut rgb_buffer) {
                    info!("failed to decode_image_to_buffer: {e}");
                    continue;
                }

                let slice = RgbSliceU8::new(&rgb_buffer, camera_dimensions);
                yuv_buffer.read_rgb8(slice);

                if frame_count % keyframe_interval == 0 {
                    h264_encoder.force_intra_frame();
                }

                let bit_stream = match h264_encoder.encode(&yuv_buffer) {
                    Ok(bit_stream) => bit_stream,
                    Err(e) => {
                        info!("failed to enocde to h264: {e}");
                        continue;
                    }
                };

                h264_encoded_buffer.clear();
                bit_stream.write_vec(&mut h264_encoded_buffer);

                openh264::nal_units(&h264_encoded_buffer)
                    .map(Bytes::copy_from_slice)
                    .for_each(|nal_unit| {
                        // TODO: remove log
                        match get_prefix_code(&nal_unit) {
                            Ok(nal_type) => info!("sending nal type: {}", nal_type as u8),
                            Err(e) => info!("{}", e),
                        }

                        match sender.send(nal_unit) {
                            Ok(()) => {}
                            Err(e) => {
                                info!("failed to send nal unit: {e}");
                            }
                        };
                    });

                let elapsed = frame_start.elapsed();
                if elapsed < Self::FRAME_DURATION {
                    std::thread::sleep(Self::FRAME_DURATION - elapsed);
                }

                frame_count += 1;
            }
        });
    }
}

impl Stream for Webcam {
    type Item = Bytes;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let receiver = match &mut self.receiver {
            Some(receiver) => receiver,
            None => return Poll::Ready(None),
        };

        match receiver.poll_recv(cx) {
            Poll::Ready(Some(data)) => Poll::Ready(Some(data)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
