use std::io::Write;

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, PrintStyledContent, Stylize},
};

use crate::{
    layout::Drawable,
    video::{
        error::Error,
        handler::{LocalVideoHandler, PeerVideoHandler, VideoHandler},
        interpolater::BilinearInterpolater,
    },
};

pub mod encoding;
pub mod error;
pub mod handler;
pub mod webcam;

mod interpolater;

const UPPER_HALF_BLOCK: char = '▀';

pub struct VideoPanel<T: VideoHandler> {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    video_handler: T,
    bilinear_interpolater: BilinearInterpolater,
}

pub type LocalVideoPanel = VideoPanel<LocalVideoHandler>;

impl LocalVideoPanel {
    pub fn new_local(x: u16, y: u16, width: u16, height: u16) -> Result<Self, Error> {
        let video_handler = LocalVideoHandler::new()?;
        Self::new(x, y, width, height, video_handler)
    }
}

pub type PeerVideoPanel = VideoPanel<PeerVideoHandler>;

impl PeerVideoPanel {
    pub fn new_peer(x: u16, y: u16, width: u16, height: u16) -> Result<Self, Error> {
        let video_handler = PeerVideoHandler::new()?;
        Self::new(x, y, width, height, video_handler)
    }
}

impl<T: VideoHandler> VideoPanel<T> {
    const PADDING: u16 = 1;

    fn new(x: u16, y: u16, width: u16, height: u16, video_handler: T) -> Result<Self, Error> {
        let bilinear_interpolater =
            BilinearInterpolater::new(width - 2 * (Self::PADDING + 1), (height - 2) * 2);

        Ok(Self {
            x: x,
            y: y,
            width: width,
            height: height,
            video_handler: video_handler,
            bilinear_interpolater: bilinear_interpolater,
        })
    }

    pub fn receive_stream(&mut self, stream: &Vec<u8>) -> Result<(), Error> {
        let (width, height) = self.video_handler.receive_stream(stream)?;
        self.bilinear_interpolater
            .update_weights_if_needed(width, height);
        self.bilinear_interpolater
            .update_grayscale_buffer(&self.video_handler.rgb_buffer());
        Ok(())
    }
}

impl<T: VideoHandler> Drawable for VideoPanel<T> {
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
