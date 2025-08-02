use std::{io::stdout, sync::Arc};

use client::{
    chat::command::Parser,
    client::{Client, signaling::init_peer_connection},
    layout::{Drawable, create_layout},
    logging::init_logging,
};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    cursor::MoveTo,
    event::{Event, EventStream},
    terminal::{self, Clear, ClearType},
};
use futures::{FutureExt, StreamExt};

use client::chat::command::ChatboxCommand;
use client::chat::command::ChatboxInput;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let mut stdout = stdout();
    stdout
        .execute(MoveTo(0, 0))?
        .execute(Clear(ClearType::All))?
        .queue(crossterm::cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let (mut chatbox, mut video_panel) = create_layout()?;
    chatbox.draw_border(&mut stdout)?;
    chatbox.draw(&mut stdout)?;
    video_panel.draw_border(&mut stdout)?;
    video_panel.draw(&mut stdout)?;

    let mut input_stream = EventStream::new();
    let client = Arc::new(Mutex::new(Client::new()));

    let mut video_stream_receiver = init_peer_connection(&client).await?;

    loop {
        let mut client_guard = client.lock().await;
        let poll_future = client_guard.poll_message();

        tokio::select! {
            Some(message) = poll_future => {
                drop(client_guard);

                let message = match message {
                    Ok(message) => message,
                    Err(e) => {
                        chatbox.error(&e.to_string());
                        chatbox.draw(&mut stdout)?;
                        continue;
                    },
                };

                let mut client_mut = client.lock().await;
                match client_mut.receive_message(&message).await {
                    Ok(_) => {},
                    Err(e) => {
                        chatbox.error(&e.to_string());
                    },
                }

                chatbox.receive_message(&message);
                chatbox.draw(&mut stdout)?;
            }

            Some(input) = input_stream.next().fuse() => {
                drop(client_guard);

                let key_event = match input {
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

                let mut client_mut = client.lock().await;
                let input_response = match client_mut.receive_input(&input).await {
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
            },

            Some(stream) = video_stream_receiver.recv() => {
                drop(client_guard);

                match video_panel.receive_stream(&stream) {
                    Ok(_) => {
                        video_panel.draw(&mut stdout)?;
                    },
                    Err(e) => {
                        chatbox.error(&e.to_string());
                        chatbox.draw(&mut stdout)?;
                        continue;
                    },
                }
            },
        }
    }

    stdout
        .execute(MoveTo(0, 0))?
        .execute(Clear(ClearType::All))?
        .execute(crossterm::cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
