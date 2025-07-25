use nokhwa::utils::Resolution;

use crate::input::webcam::Webcam;

pub mod keyboard;
pub mod webcam;

pub enum InputType<'a> {
    Normal { character: char },
    Backspace,
    Send,
    Webcam { webcam: &'a mut Webcam },
    Exit,
    CreateRoom,
    JoinRoom,
    Unknown,
}

pub trait VideoFeed {
    fn write_next_frame(&mut self, buffer: &mut [u8]) -> Result<(), Box<dyn std::error::Error>>;
    fn resolution(&self) -> Resolution;
}
