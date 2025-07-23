use std::time::Duration;

use crate::{
    input::{InputType, webcam::Webcam},
    ui::screen::Screen,
};

mod chatbox;
mod screen;
mod tile;
mod video;

pub const FRAME_DURATION: Duration = Duration::from_millis(30);

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
