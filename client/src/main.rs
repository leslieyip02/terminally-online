use std::io::stdout;

use client::{chat::command::Parser, client::Client};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    cursor::MoveTo,
    event::{Event, EventStream},
    terminal::{self, Clear, ClearType},
};
use futures::{FutureExt, StreamExt};

use client::chat::command::ChatboxInput;
use client::{
    chat::{Chatbox, command::ChatboxCommand},
    ui::{Drawable, FRAME_DURATION},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    stdout
        .execute(MoveTo(0, 0))?
        .execute(Clear(ClearType::All))?
        .queue(crossterm::cursor::Hide)?;
    terminal::enable_raw_mode()?;

    // TODO: replace temporary code
    let size = match termsize::get() {
        Some(size) => size,
        None => panic!("Unable to get terminal size."),
    };
    let mut chatbox = Chatbox::new(1, 1, size.cols - 2, size.rows - 2);
    chatbox.draw_border(&mut stdout)?;
    chatbox.draw(&mut stdout)?;

    let mut input_stream = EventStream::new();
    let mut interval = tokio::time::interval(FRAME_DURATION);
    let mut client = Client::new();
    client.init().await?;

    loop {
        tokio::select! {
            Some(event) = input_stream.next().fuse() => {
                let key_event = match event {
                    Ok(Event::Key(key_event)) => key_event,
                    _ => continue,
                };

                let input = match chatbox.input(&key_event) {
                    Ok(input) => input,
                    Err(e) => {
                        chatbox.error(&e.to_string());
                        chatbox.draw(&mut stdout)?;
                        continue;
                    },
                };

                match &input {
                    ChatboxInput::Command(command) => match command {
                        ChatboxCommand::Exit => break,
                        _ => {},
                    },
                    ChatboxInput::Exit => break,
                    _ => {},
                }

                let input_response = match client.receive_input(&input).await {
                    Ok(input_response) => input_response,
                    Err(e) => {
                        chatbox.error(&e.to_string());
                        chatbox.draw(&mut stdout)?;
                        continue;
                    },
                };

                match input_response {
                    Some(log) => chatbox.log(&log),
                    None => {},
                }

                chatbox.draw(&mut stdout)?;
            }

            Some(message) = client.next() => {
                let message = match message {
                    Ok(message) => message,
                    Err(e) => {
                        chatbox.error(&e.to_string());
                        chatbox.draw(&mut stdout)?;
                        continue;
                    },
                };

                chatbox.receive_message(&message);
                chatbox.draw(&mut stdout)?;
            }

            _ = interval.tick() => {},
        }
    }

    stdout
        .execute(MoveTo(0, 0))?
        .execute(Clear(ClearType::All))?
        .execute(crossterm::cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
