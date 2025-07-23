use crate::{
    input::InputType,
    ui::{
        Updatable,
        screen::{ScreenComponent, ScreenOrigin},
        tile::Tile,
    },
};

pub struct Chatbox {
    sent_messages: Vec<String>,
    buffered_message: String,
    display_width: usize,
    display_height: usize,
    origin: ScreenOrigin,
}

impl Chatbox {
    fn write_sent_messages_to_screen(&self, screen_buffer: &mut Vec<Tile>) {
        self.sent_messages.iter().fold(0, |acc, message| {
            let wrapped = textwrap::wrap(message, self.display_width);
            wrapped.iter().enumerate().for_each(|(y, line)| {
                let origin_begin = (self.origin.y + y + acc) * self.origin.stride + self.origin.x;
                line.chars().enumerate().for_each(|(x, character)| {
                    screen_buffer[origin_begin + x].content = character;
                });
            });

            acc + wrapped.len()
        });
    }

    fn write_buffered_message_to_screen(&self, screen_buffer: &mut Vec<Tile>) {
        let wrapped = textwrap::wrap(&self.buffered_message, self.display_width);
        wrapped.iter().enumerate().for_each(|(y, line)| {
            let origin_begin =
                (self.origin.y + self.display_height - 3 + y) * self.origin.stride + self.origin.x;
            line.chars().enumerate().for_each(|(x, character)| {
                screen_buffer[origin_begin + x].content = character;
            });
        });
    }

    fn clear_screen(&self, screen_buffer: &mut Vec<Tile>) {
        (0..self.display_height).for_each(|y| {
            let origin_begin = (self.origin.y + y) * self.origin.stride + self.origin.x;
            (0..self.display_width).for_each(|x| {
                screen_buffer[origin_begin + x].reset();
            });
        });
    }
}

impl ScreenComponent for Chatbox {
    fn new(display_width: usize, display_height: usize, origin: ScreenOrigin) -> Self {
        Self {
            sent_messages: vec![],
            buffered_message: String::new(),
            display_width: display_width,
            display_height: display_height,
            origin: origin,
        }
    }

    fn write_to_screen(&self, screen_buffer: &mut Vec<Tile>) {
        self.clear_screen(screen_buffer);
        self.write_sent_messages_to_screen(screen_buffer);
        self.write_buffered_message_to_screen(screen_buffer);
    }
}

impl Updatable for Chatbox {
    fn update(&mut self, input: InputType) -> Result<(), Box<dyn std::error::Error>> {
        match input {
            InputType::Normal { character } => self.buffered_message.push(character),
            InputType::Backspace => {
                self.buffered_message.pop();
            }
            InputType::Send => {
                self.sent_messages.push(self.buffered_message.clone());
                self.buffered_message.clear();
            }
            _ => {}
        }

        Ok(())
    }
}
