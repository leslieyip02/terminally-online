use crossterm::event::{KeyCode, KeyEvent};

use crate::input::InputType;

const QUIT_COMMAND: &str = "/quit";

pub struct Keyboard {
    buffer: String,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn input(&mut self, event: &KeyEvent) -> Result<InputType, std::io::Error> {
        match event.code {
            KeyCode::Char(c) => {
                self.buffer.push(c);
                return Ok(InputType::Normal { character: c });
            }
            KeyCode::Backspace => {
                self.buffer.pop();
                return Ok(InputType::Backspace);
            }
            KeyCode::Enter => match self.buffer.as_str().trim() {
                QUIT_COMMAND => return Ok(InputType::Exit),
                _ => return Ok(InputType::Send),
            },
            KeyCode::Esc => return Ok(InputType::Exit),
            _ => return Ok(InputType::Unknown),
        }
    }
}
