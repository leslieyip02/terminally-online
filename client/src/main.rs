use std::io::stdout;

use client::room::client::RoomClient;
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
    let mut room_client = RoomClient::new();

    loop {
        tokio::select! {
            Some(event) = input_stream.next().fuse() => {
                let key_event = match event {
                    Ok(Event::Key(key_event)) => key_event,
                    _ => continue,
                };

                match chatbox.input(&key_event)? {
                    ChatboxInput::Message(content) => {
                        if let Err(_) = room_client.send_chat_message(&content).await {
                            chatbox.receive_error("unable to send message");
                        }
                    },
                    ChatboxInput::Command(command) => {
                        match command {
                            ChatboxCommand::Create => {
                                match room_client.create_and_connect_to_room().await {
                                    Ok(room_id) => {
                                        // TODO: fix hack
                                        chatbox.receive_error(&format!("id = {}", room_id));
                                    },
                                    Err(_) => {
                                        chatbox.receive_error("unable to create room");
                                    }
                                }
                            },
                            ChatboxCommand::Join {room_id} => {
                                if let Err(e) = room_client.join_and_connect_to_room(&room_id).await {
                                    let error = format!("unable to join room {}: {}", &room_id, e);
                                    chatbox.receive_error(&error);
                                }
                            },
                            ChatboxCommand::Quit => break,
                        }
                    },
                    ChatboxInput::Error(error) => {
                        chatbox.receive_error(&error);
                    }
                    ChatboxInput::Exit => break,
                    ChatboxInput::None => {},
                }
                chatbox.draw(&mut stdout)?;
            },
            Some(message) = room_client.next() => {
                let message = match message {
                    Ok(message) => message,
                    Err(_) => {
                        let error = format!("failed to receive message");
                        chatbox.receive_error(&error);
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
