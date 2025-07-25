use std::{io::Stdout, time::Duration};

use crossterm::{ExecutableCommand, QueueableCommand, cursor::MoveTo, style::Print};

use crate::{
    input::{InputType, webcam::Webcam},
    ui::screen::Screen,
};

mod screen;
mod tile;
mod video;

pub const FRAME_DURATION: Duration = Duration::from_millis(30);

pub trait Drawable {
    const TOP_LEFT_CORNER: &str = "╭";
    const TOP_RIGHT_CORNER: &str = "╮";
    const BOTTOM_LEFT_CORNER: &str = "╰";
    const BOTTOM_RIGHT_CORNER: &str = "╯";
    const HORIZONTAL_BORDER: &str = "─";
    const VERTICAL_BORDER: &str = "│";

    fn draw(&self, stdout: &mut std::io::Stdout) -> Result<(), std::io::Error>;

    fn x(&self) -> u16;
    fn y(&self) -> u16;
    fn width(&self) -> u16;
    fn height(&self) -> u16;

    fn draw_border(&self, stdout: &mut std::io::Stdout) -> Result<(), std::io::Error> {
        let x = self.x();
        let y = self.y();
        let w = self.width();
        let h = self.height();

        stdout
            .queue(MoveTo(x, y))?
            .queue(Print(Self::TOP_LEFT_CORNER))?;
        stdout
            .queue(MoveTo(x + w - 1, y))?
            .queue(Print(Self::TOP_RIGHT_CORNER))?;
        stdout
            .queue(MoveTo(x, y + h - 1))?
            .queue(Print(Self::BOTTOM_LEFT_CORNER))?;
        stdout
            .queue(MoveTo(x + w - 1, y + h - 1))?
            .queue(Print(Self::BOTTOM_RIGHT_CORNER))?;

        for i in 1..w - 1 {
            stdout
                .queue(MoveTo(x + i, y))?
                .queue(Print(Self::HORIZONTAL_BORDER))?
                .queue(MoveTo(x + i, y + h - 1))?
                .queue(Print(Self::HORIZONTAL_BORDER))?;
        }
        for i in 1..h - 1 {
            stdout
                .queue(MoveTo(x, y + i))?
                .queue(Print(Self::VERTICAL_BORDER))?
                .queue(MoveTo(x + w - 1, y + i))?
                .queue(Print(Self::VERTICAL_BORDER))?;
        }

        Ok(())
    }
}

pub trait Updatable {
    fn update(&mut self, input: InputType) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Printable {
    fn print(&self, out: &mut dyn std::io::Write) -> Result<(), std::io::Error>;
}

pub struct Ui {
    screen: Screen,
}

impl Ui {
    pub fn new(webcam: &Webcam) -> Result<Self, Box<dyn std::error::Error>> {
        let size = match termsize::get() {
            Some(size) => size,
            None => panic!("Unable to get terminal size."),
        };
        let screen = Screen::new(size.cols as usize, size.rows as usize, webcam)?;
        Ok(Self { screen: screen })
    }
}

impl Updatable for Ui {
    fn update(&mut self, input: InputType) -> Result<(), Box<dyn std::error::Error>> {
        self.screen.update(input)
    }
}

impl Printable for Ui {
    fn print(&self, out: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        self.screen.print(out)
    }
}
