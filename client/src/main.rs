use std::{io::stdout, sync::Arc};

use client::{
    chat::command::Parser,
    client::{
        Client,
        signaling::{init_peer_connection, send_video},
    },
};
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
use tokio::sync::Mutex;
use tracing_appender::rolling;

fn init_logging() {
    // Log file will rotate daily under ./logs/
    let file_appender = rolling::daily("logs", "app.log");

    let subscriber = tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false) // no colors in file
        .with_thread_names(true)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

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
    let client = Arc::new(Mutex::new(Client::new()));
    init_peer_connection(&client).await?;

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
                        // TODO: temporary for testing
                        ChatboxCommand::Stream => {
                            send_video(&client).await?;
                        },
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
            }

            _ = interval.tick() => {
                drop(client_guard);
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
