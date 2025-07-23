use crossterm::{
    QueueableCommand,
    cursor::{self, MoveToNextLine},
    style::{self},
};
use itertools::Itertools;

use crate::{
    input::InputType,
    ui::{Printable, Updatable, chatbox::Chatbox, tile::Tile, video::Video},
};

pub struct Screen {
    screen_buffer: Vec<Tile>,
    screen_width: usize,
    video: Video,
    chatbox: Chatbox,
}

pub trait ScreenComponent {
    fn new(display_width: usize, display_height: usize, origin: ScreenOrigin) -> Self;
    fn write_to_screen(&self, screen_buffer: &mut Vec<Tile>);
}

pub struct ScreenOrigin {
    pub x: usize,
    pub y: usize,
    pub stride: usize,
}

impl Screen {
    pub fn new(
        screen_width: usize,
        screen_height: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let buffer_size = screen_width * screen_height;

        // TODO: replace placeholder values
        let video = Video::new(
            128,
            72,
            ScreenOrigin {
                x: 2,
                y: 1,
                stride: screen_width,
            },
        );
        let chatbox = Chatbox::new(
            screen_width - video.width() - 4,
            screen_height - 2,
            ScreenOrigin {
                x: video.width() + 4,
                y: 1,
                stride: screen_width,
            },
        );

        Ok(Self {
            screen_buffer: vec![Tile::default(); buffer_size as usize],
            screen_width: screen_width,
            video: video,
            chatbox: chatbox,
        })
    }
}

impl Updatable for Screen {
    fn update(&mut self, input: InputType) -> Result<(), Box<dyn std::error::Error>> {
        match input {
            InputType::TickUpdate => {
                self.video.update(input)?;
                self.video.write_to_screen(&mut self.screen_buffer);
            }
            _ => {
                self.chatbox.update(input)?;
                self.chatbox.write_to_screen(&mut self.screen_buffer);
            }
        }

        Ok(())
    }
}

impl Printable for Screen {
    fn print(&self, out: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        out.queue(cursor::MoveTo(0, 0))?;

        self.screen_buffer
            .iter()
            .chunks(self.screen_width)
            .into_iter()
            .try_for_each(|mut line| {
                line.try_for_each(|tile| {
                    let content = tile.styled();
                    out.queue(style::PrintStyledContent(content))?;
                    Ok::<_, std::io::Error>(())
                })?;
                out.queue(MoveToNextLine(1))?;
                Ok::<_, std::io::Error>(())
            })?;

        Ok(())
    }
}
