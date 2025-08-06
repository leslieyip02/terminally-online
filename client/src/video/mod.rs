use std::io::Write;

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, PrintStyledContent, Stylize},
};
use tracing::info;

use crate::{
    layout::Drawable,
    video::{
        error::Error,
        handler::{PeerVideoHandler, VideoHandler},
        interpolater::BilinearInterpolater,
    },
};

pub mod encoding;
pub mod error;
pub mod handler;
pub mod webcam;

mod interpolater;

const UPPER_HALF_BLOCK: char = '▀';

// VideoPanel -> receive stream, draw, update BilinearInterpolater?
// VideoRenderer -> draw using BilinearInterpolater

// TODO: take a dynamic VideoHandler
pub struct VideoPanel {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    video_handler: Box<dyn VideoHandler>,
    bilinear_interpolater: BilinearInterpolater,
}

impl VideoPanel {
    const PADDING: u16 = 1;

    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Result<Self, Error> {
        let handler = PeerVideoHandler::new()?;
        let bilinear_interpolater =
            BilinearInterpolater::new(width - 2 * (Self::PADDING + 1), (height - 2) * 2);

        Ok(Self {
            x: x,
            y: y,
            width: width,
            height: height,
            video_handler: Box::new(handler),
            bilinear_interpolater: bilinear_interpolater,
        })
    }

    pub fn receive_stream(&mut self, stream: &Vec<u8>) -> Result<(), Error> {
        let (width, height) = self.video_handler.receive_stream(stream)?;
        info!("received");
        self.bilinear_interpolater
            .update_weights_if_needed(width, height);
        self.bilinear_interpolater
            .update_grayscale_buffer(&self.video_handler.rgb_buffer());
        info!("updated");
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
