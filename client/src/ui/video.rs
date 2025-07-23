use itertools::Itertools;
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{RequestedFormat, RequestedFormatType},
};

use crate::{
    input::InputType,
    ui::{
        Updatable,
        screen::{ScreenComponent, ScreenOrigin},
        tile::Tile,
    },
};

pub struct InterpolationWeight {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    dx: f32,
    dy: f32,
}

impl InterpolationWeight {
    fn new(x: usize, y: usize, x_ratio: f32, y_ratio: f32) -> Self {
        let x = x as f32;
        let y = y as f32;

        let x0 = (x * x_ratio).floor() as usize;
        let y0 = (y * y_ratio).floor() as usize;
        let x1 = (x * x_ratio).ceil() as usize;
        let y1 = (y * y_ratio).ceil() as usize;
        let dx = (x * x_ratio) - x0 as f32;
        let dy = (y * y_ratio) - y0 as f32;

        InterpolationWeight {
            x0: x0,
            y0: y0,
            x1: x1,
            y1: y1,
            dx: dx,
            dy: dy,
        }
    }
}

pub struct Video {
    camera: Camera,
    input_buffer: Vec<u8>,
    input_width: usize,
    display_width: usize,
    interpolation_weights: Vec<InterpolationWeight>,
    origin: ScreenOrigin,
}

impl Video {
    pub fn width(&self) -> usize {
        self.display_width
    }

    fn at(&self, x: usize, y: usize) -> f32 {
        let r = self.input_buffer[y * self.input_width * 3 + x * 3] as f32;
        let g = self.input_buffer[y * self.input_width * 3 + x * 3 + 1] as f32;
        let b = self.input_buffer[y * self.input_width * 3 + x * 3 + 2] as f32;

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn interpolate(&self, weight: &InterpolationWeight) -> u8 {
        let a = self.at(weight.x0, weight.y0) * (1.0 - weight.dx) * (1.0 - weight.dy);
        let b = self.at(weight.x1, weight.y0) * weight.dx * (1.0 - weight.dy);
        let c = self.at(weight.x0, weight.y1) * (1.0 - weight.dx) * weight.dy;
        let d = self.at(weight.x1, weight.y1) * weight.dx * weight.dy;

        (a + b + c + d).clamp(0.0, 255.0) as u8
    }

    fn compute_interpolation_weights(
        input_width: usize,
        input_height: usize,
        display_width: usize,
        display_height: usize,
    ) -> Vec<InterpolationWeight> {
        let x_ratio = (input_width - 1) as f32 / (display_width) as f32;
        let y_ratio = (input_height - 1) as f32 / (display_height) as f32;

        (0..display_height)
            .flat_map(|y| (0..display_width).map(move |x| (x, y)))
            .map(|(x, y)| InterpolationWeight::new(x, y, x_ratio, y_ratio))
            .collect()
    }
}

impl ScreenComponent for Video {
    fn new(display_width: usize, display_height: usize, origin: ScreenOrigin) -> Self {
        // TODO: take any video feed input
        let index = nokhwa::utils::CameraIndex::Index(0);
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
        let mut camera = Camera::new(index, format).unwrap();
        camera.open_stream().unwrap();

        let input_width = camera.resolution().width() as usize;
        let input_height = camera.resolution().height() as usize;
        let input_buffer = vec![0; input_width * input_height * 3];
        let interpolation_weights = Video::compute_interpolation_weights(
            input_width,
            input_height,
            display_width,
            display_height,
        );

        Self {
            camera: camera,
            input_buffer: input_buffer,
            input_width: input_width,
            display_width: display_width,
            interpolation_weights: interpolation_weights,
            origin: origin,
        }
    }

    fn write_to_screen(&self, screen_buffer: &mut Vec<Tile>) {
        self.interpolation_weights
            .iter()
            .map(|weight| self.interpolate(weight))
            .chunks(self.display_width)
            .into_iter()
            .enumerate()
            .for_each(|(y, values)| {
                let origin_begin = (self.origin.y + y / 2) * self.origin.stride + self.origin.x;
                let is_top = y % 2 == 0;
                values.enumerate().for_each(|(x, value)| {
                    let index = origin_begin + x;
                    if is_top {
                        screen_buffer[index].top_brightness = value;
                    } else {
                        screen_buffer[index].bottom_brightness = value;
                    }
                });
            });
    }
}

impl Updatable for Video {
    fn update(&mut self, input: InputType) -> Result<(), Box<dyn std::error::Error>> {
        match input {
            InputType::TickUpdate => {
                let frame = self.camera.frame()?;
                frame.decode_image_to_buffer::<RgbFormat>(&mut self.input_buffer)?;
            }
            _ => {}
        }

        Ok(())
    }
}
