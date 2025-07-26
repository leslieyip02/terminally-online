use std::io::stdout;

use client::{
    chat::command::Parser,
    client::{Client, signaling::ConnectionManager},
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
    let mut connection_manager = ConnectionManager::new().await?;

    // TODO: move into client?
    let mut is_creator = false;

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
                        continue;
                    },
                };

                match input {
                    ChatboxInput::Message(content) => {
                        if let Err(e) = client.send_chat_message(&content).await {
                            chatbox.error(&e.to_string());
                        }
                    },
                    ChatboxInput::Command(command) => {
                        match command {
                            ChatboxCommand::Create => {
                                match client.create_and_connect_to_room().await {
                                    Ok(room_id) => {
                                        chatbox.log(&format!("room id = {}", room_id));
                                        is_creator = true;
                                    },
                                    Err(e) => {
                                        chatbox.error(&e.to_string());
                                    }
                                }
                            },
                            ChatboxCommand::Join {room_id} => {
                                if let Err(e) = client.join_and_connect_to_room(&room_id).await {
                                    chatbox.error(&e.to_string());
                                }
                            },
                            ChatboxCommand::Exit => break,
                        }
                    },
                    ChatboxInput::Exit => break,
                    ChatboxInput::None => {},
                }
                chatbox.draw(&mut stdout)?;
            },
            Some(message) = client.next() => {
                let message = match message {
                    Ok(message) => message,
                    Err(e) => {
                        chatbox.error(&e.to_string());
                        chatbox.draw(&mut stdout)?;
                        continue;
                    },
                };

                match connection_manager.receive_message(&message, &mut client, is_creator).await {
                    Ok(_) => {},
                    Err(e) => chatbox.error(&e.to_string()),
                }

                chatbox.receive_message(&message);

                // // TODO: hack
                // if is_creator {
                //     match message {
                //         Message::Room { room_message } => {
                //             match room_message {
                //                 RoomMessage::Join { .. } => {
                //                     connection_manager.create_offer(&mut room_client).await?;
                //                 },
                //                 _ => {},
                //             }
                //         },
                //         _ => {},
                //     }
                // }

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
