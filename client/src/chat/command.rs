use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent};

use crate::chat::Chatbox;

pub enum ChatboxCommand {
    Create,
    Join { room_id: String },
    Quit,
}

pub enum ChatboxInput {
    Message(String),
    Command(ChatboxCommand),
    Exit,
    None,
}

const COMMAND_MARKER: &str = "/";
const CREATE_COMMAND: &str = "/create";
const JOIN_COMMAND: &str = "/join";
const QUIT_COMMAND: &str = "/quit";

fn parse(input: &str) -> ChatboxInput {
    if input.starts_with(COMMAND_MARKER) {
        parse_command(input)
    } else {
        parse_message(input)
    }
}

fn parse_message(input: &str) -> ChatboxInput {
    ChatboxInput::Message(String::from(input))
}

fn parse_command(input: &str) -> ChatboxInput {
    let tokens = input
        .split_whitespace()
        .map(|token| token.to_string())
        .collect::<VecDeque<String>>();

    let command = match tokens.front() {
        Some(command) => command,
        None => return ChatboxInput::None,
    };

    let command = match command.as_str() {
        CREATE_COMMAND => ChatboxCommand::Create,
        JOIN_COMMAND => ChatboxCommand::Join {
            room_id: tokens[1].clone(),
        },
        QUIT_COMMAND => ChatboxCommand::Quit,
        _ => return ChatboxInput::None,
    };
    ChatboxInput::Command(command)
}

impl Chatbox {
    pub fn input(&mut self, key_event: &KeyEvent) -> Result<ChatboxInput, std::io::Error> {
        match key_event.code {
            KeyCode::Char(c) => {
                self.typing_buffer.push(c);
                return Ok(ChatboxInput::None);
            }
            KeyCode::Backspace => {
                self.typing_buffer.pop();
                return Ok(ChatboxInput::None);
            }
            KeyCode::Enter => {
                // TODO: remove debug
                self.receive_message(self.typing_buffer.clone());
                let response = parse(&self.typing_buffer);
                self.typing_buffer.clear();
                return Ok(response);
            }
            KeyCode::Esc => return Ok(ChatboxInput::Exit),
            _ => return Ok(ChatboxInput::None),
        }
    }
}
