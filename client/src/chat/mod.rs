use std::{collections::VecDeque, io::Write};

use crossterm::{QueueableCommand, cursor::MoveTo, style::Print};

use crate::ui::Drawable;

pub mod command;

pub struct Chatbox {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    lines_buffer: VecDeque<String>,
    typing_buffer: String,
}

impl Chatbox {
    const PADDING: u16 = 1;
    const DIVIDER: &str = "─";
    const SPACE: &str = " ";
    const TYPING_INDICATOR: &str = ">";

    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        let lines_buffer = VecDeque::with_capacity(height as usize - 2);
        let typing_buffer = String::new();

        Self {
            x: x,
            y: y,
            width: width,
            height: height,
            lines_buffer: lines_buffer,
            typing_buffer: typing_buffer,
        }
    }

    fn line_width(&self) -> usize {
        self.width as usize - 2 * (Self::PADDING + 1) as usize
    }

    pub fn receive_message(&mut self, message: String) {
        textwrap::wrap(&message, self.line_width())
            .iter()
            .for_each(|line| {
                self.lines_buffer.push_back(line.to_string());
            });

        while self.lines_buffer.len() > self.height as usize {
            self.lines_buffer.pop_front();
        }
    }
}

impl Drawable for Chatbox {
    fn draw(&self, stdout: &mut std::io::Stdout) -> Result<(), std::io::Error> {
        for i in 1..self.height - 1 {
            stdout
                .queue(MoveTo(self.x + Self::PADDING + 1, self.y + i as u16))?
                .queue(Print(Self::SPACE.repeat(self.line_width())))?;
        }

        let typing_buffer = format!("{} {}", Self::TYPING_INDICATOR, &self.typing_buffer);
        let typing_buffer_lines = textwrap::wrap(&typing_buffer, self.line_width());
        let typing_buffer_height = typing_buffer_lines.len() as u16;
        for (i, line) in typing_buffer_lines.into_iter().enumerate() {
            stdout
                .queue(MoveTo(
                    self.x + Self::PADDING + 1,
                    self.y + self.height - typing_buffer_height + i as u16 - 1,
                ))?
                .queue(Print(line))?;
        }

        let divider_y = self.y + self.height - typing_buffer_height - 2;
        stdout
            .queue(MoveTo(self.x + Self::PADDING + 1, divider_y))?
            .queue(Print(Self::DIVIDER.repeat(self.line_width())))?;

        let available = divider_y as usize - 2;
        let lines_buffer_size = self.lines_buffer.len();
        let (start_line, num_lines) = if available < lines_buffer_size {
            (lines_buffer_size - available, available)
        } else {
            (0, lines_buffer_size)
        };
        for i in 0..num_lines {
            stdout
                .queue(MoveTo(self.x + Self::PADDING + 1, self.y + i as u16 + 1))?
                .queue(Print(&self.lines_buffer[start_line + i]))?;
        }

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
