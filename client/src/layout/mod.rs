use crossterm::{cursor::MoveTo, style::Print, QueueableCommand};

use crate::{chat::Chatbox, video::VideoPanel};

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

pub fn create_layout() -> Result<(Chatbox, VideoPanel), Box<dyn std::error::Error>> {
    let size = match termsize::get() {
        Some(size) => size,
        None => panic!("Unable to get terminal size."),
    };

    let width = size.cols;
    let height = size.rows;

    // TODO: improve this
    if width < height {
        Ok((
            Chatbox::new(1, 34, width - 2, height - 34),
            VideoPanel::new(1, 1, width - 2, 32)?,
        ))
    } else {
        Ok((
            Chatbox::new(130, 1, width - 128 - 4, height - 2),
            VideoPanel::new(1, 1, 128, height - 2)?,
        ))
    }
}
