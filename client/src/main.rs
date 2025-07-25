use std::io::{Write, stdout};

use crossterm::{
    ExecutableCommand, QueueableCommand,
    cursor::MoveTo,
    event::{Event, EventStream},
    terminal::{self, Clear, ClearType},
};
use futures::{FutureExt, StreamExt};
use reqwest::Client;

use client::{chat::command::ChatboxInput, room::join_room};
use client::{
    chat::{Chatbox, command::ChatboxCommand},
    input::{InputType, webcam::Webcam},
    room::{connect_to_room, create_room},
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
                                create_room(&client).await?;
                            },
                            ChatboxCommand::Join {room_id} => {
                                join_room(&client, &room_id).await?;
                            },
                            ChatboxCommand::Quit => break,
                        }
                    },
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
