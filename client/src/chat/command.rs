use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent};

use crate::chat::{Chatbox, error::Error};

pub enum ChatboxCommand {
    Create,
    Join { room_id: String },
    Exit,
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
const EXIT_COMMAND: &str = "/exit";
const QUIT_COMMAND: &str = "/quit";

fn parse_message(input: &str) -> Result<ChatboxInput, Error> {
    Ok(ChatboxInput::Message(String::from(input)))
}

fn parse_command(input: &str) -> Result<ChatboxInput, Error> {
    let tokens = input
        .split_whitespace()
        .map(|token| token.to_string())
        .collect::<VecDeque<String>>();

    let command = match tokens.front() {
        Some(command) => command,
        None => return Err(Error::InvalidCommand),
    };

    let command = match command.as_str() {
        CREATE_COMMAND => ChatboxCommand::Create,
        JOIN_COMMAND => {
            if tokens.len() < 2 {
                return Err(Error::InvalidUsage {
                    usage: String::from("/join <room_id>"),
                });
            }

            ChatboxCommand::Join {
                room_id: tokens[1].clone(),
            }
        }
        EXIT_COMMAND => ChatboxCommand::Exit,
        QUIT_COMMAND => ChatboxCommand::Exit,
        _ => return Err(Error::InvalidCommand),
    };

    Ok(ChatboxInput::Command(command))
}

fn parse(input: &str) -> Result<ChatboxInput, Error> {
    if input.starts_with(COMMAND_MARKER) {
        parse_command(input)
    } else {
        parse_message(input)
    }
}

pub trait Parser {
    fn input(&mut self, key_event: &KeyEvent) -> Result<ChatboxInput, Error>;
}

impl Parser for Chatbox {
    fn input(&mut self, key_event: &KeyEvent) -> Result<ChatboxInput, Error> {
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
                let input = self.typing_buffer.clone();
                self.typing_buffer.clear();
                parse(&input)
            }
            KeyCode::Esc => return Ok(ChatboxInput::Exit),
            _ => return Ok(ChatboxInput::None),
        }
    }
}
