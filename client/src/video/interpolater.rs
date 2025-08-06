struct BilinearWeight {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    dx: f32,
    dy: f32,
}

impl BilinearWeight {
    fn new(x: f32, y: f32, x_ratio: f32, y_ratio: f32) -> Self {
        let x0 = (x * x_ratio).floor();
        let y0 = (y * y_ratio).floor();

        BilinearWeight {
            x0: x0 as usize,
            y0: y0 as usize,
            x1: (x * x_ratio).ceil() as usize,
            y1: (y * y_ratio).ceil() as usize,
            dx: (x * x_ratio) - x0,
            dy: (y * y_ratio) - y0,
        }
    }
}

pub(crate) struct BilinearInterpolater {
    display_width: u16,
    display_height: u16,
    input_width: usize,
    input_height: usize,
    bilinear_weights: Vec<BilinearWeight>,
    grayscale_buffer: Vec<u8>,
}

impl BilinearInterpolater {
    pub(crate) fn new(display_width: u16, display_height: u16) -> Self {
        Self {
            display_width: display_width,
            display_height: display_height,
            input_width: 0,
            input_height: 0,
            bilinear_weights: Vec::new(),
            grayscale_buffer: vec![0; display_width as usize * display_height as usize],
        }
    }

    pub(crate) fn update_weights_if_needed(&mut self, input_width: usize, input_height: usize) {
        if input_width == self.input_width && input_height == self.input_height {
            return;
        }

        let x_ratio = (input_width - 1) as f32 / (self.display_width) as f32;
        let y_ratio = (input_height - 1) as f32 / (self.display_height) as f32;

        self.input_width = input_width;
        self.input_height = input_height;
        self.bilinear_weights = (0..self.display_height)
            .flat_map(|y| (0..self.display_width).map(move |x| (x, y)))
            .map(|(x, y)| BilinearWeight::new(x as f32, y as f32, x_ratio, y_ratio))
            .collect()
    }

    pub(crate) fn update_grayscale_buffer(&mut self, rgb_buffer: &[u8]) {
        self.bilinear_weights
            .iter()
            .enumerate()
            .for_each(|(index, weight)| {
                self.grayscale_buffer[index] = interpolate(self.input_width, rgb_buffer, weight)
            });
    }

    pub(crate) fn grouped_rows(&self) -> impl Iterator<Item = &[u8]> {
        self.grayscale_buffer
            .chunks(self.display_width as usize * 2)
    }
}

fn at(x: usize, y: usize, stride: usize, rgb_buffer: &[u8]) -> f32 {
    let r = rgb_buffer[y * stride * 3 + x * 3] as f32;
    let g = rgb_buffer[y * stride * 3 + x * 3 + 1] as f32;
    let b = rgb_buffer[y * stride * 3 + x * 3 + 2] as f32;

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn interpolate(stride: usize, rgb_buffer: &[u8], weight: &BilinearWeight) -> u8 {
    let a = at(weight.x0, weight.y0, stride, rgb_buffer) * (1.0 - weight.dx) * (1.0 - weight.dy);
    let b = at(weight.x1, weight.y0, stride, rgb_buffer) * weight.dx * (1.0 - weight.dy);
    let c = at(weight.x0, weight.y1, stride, rgb_buffer) * (1.0 - weight.dx) * weight.dy;
    let d = at(weight.x1, weight.y1, stride, rgb_buffer) * weight.dx * weight.dy;

    (a + b + c + d).clamp(0.0, 255.0) as u8
}
