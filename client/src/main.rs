use std::io::{Write, stdout};
use std::time::Duration;

use client::room::{create_room_with_timeout, join_room_with_timeout};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    cursor::MoveTo,
    event::{Event, EventStream},
    terminal::{self, Clear, ClearType},
};
use futures::{FutureExt, StreamExt};
use reqwest::Client;

use client::chat::command::ChatboxInput;
use client::{
    chat::{Chatbox, command::ChatboxCommand},
    input::{InputType, webcam::Webcam},
    ui::{Drawable, FRAME_DURATION, Printable, Ui},
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

    // let mut webcam = Webcam::new()?;
    // let mut keyboard = Keyboard::new();
    // let mut ui = Ui::new(&webcam)?;

    let mut input_stream = EventStream::new();
    let mut interval = tokio::time::interval(FRAME_DURATION);

    let client = Client::new();

    loop {
        tokio::select! {
            Some(event) = input_stream.next().fuse() => {
                let key_event = match event {
                    Ok(Event::Key(key_event)) => key_event,
                    _ => continue,
                };

                match chatbox.input(&key_event)? {
                    // TODO: implement
                    ChatboxInput::Message(_) => {},
                    ChatboxInput::Command(command) => {
                        match command {
                            ChatboxCommand::Create => {
                                match create_room_with_timeout(&client).await {
                                    Ok(_) => {},
                                    Err(_) => {
                                        chatbox.receive_error("unable to create room");
                                    },
                                }
                            },
                            ChatboxCommand::Join {room_id} => {
                                match join_room_with_timeout(&client, &room_id).await {
                                    Ok(_) => {},
                                    Err(_) => {
                                        let message = format!("unable to join room {}", &room_id);
                                        chatbox.receive_error(&message);
                                    },
                                }
                            },
                            ChatboxCommand::Quit => break,
                        }
                    },
                    ChatboxInput::Error(message) => {
                        chatbox.receive_error(&message);
                    }
                    ChatboxInput::Exit => break,
                    ChatboxInput::None => {},
                }
                chatbox.draw(&mut stdout)?;

                // match event {
                //     Some(Ok(Event::Key(event))) => {
                //         match keyboard.input(&event)? {
                //             InputType::Exit => break,
                //             input => {
                //                 ui.update(input)?;
                //             },
                //         }
                //     },
                //     _ => continue,
                // }
            },
            _ = interval.tick() => {},
            // // TODO: refactor
            // event = input_stream.next().fuse() => {
            //     match event {
            //         Some(Ok(Event::Key(event))) => {
            //             match keyboard.input(&event)? {
            //                 InputType::Exit => break,
            //                 input => {
            //                     ui.update(input)?;
            //                 },
            //             }
            //         },
            //         _ => continue,
            //     }
            // },
            // _ = interval.tick() => {
            //     ui.update(InputType::Webcam { webcam: &mut webcam })?;
            //     ui.print(&mut s)?
            // }
        }
    }

    stdout
        .execute(MoveTo(0, 0))?
        .execute(Clear(ClearType::All))?
        .execute(crossterm::cursor::Show)?;
    terminal::disable_raw_mode()?;

    // let client = Client::new();
    // let response = create_room(&client).await?;
    // println!("Created room: {}", response.room);

    // println!("Received token: {}", response.token);
    // connect_to_room(&response.token).await?;

    Ok(())
}
