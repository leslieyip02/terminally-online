use std::io::stdout;

use crossterm::{
    QueueableCommand,
    event::{Event, EventStream},
    terminal,
};
use futures::{FutureExt, StreamExt};
use reqwest::Client;

use client::{
    input::{keyboard::Keyboard, webcam::Webcam, InputType},
    room::{connect_to_room, create_room},
    ui::{Printable, Ui, Updatable, FRAME_DURATION},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut webcam = Webcam::new()?;
    // let mut keyboard = Keyboard::new();
    // let mut ui = Ui::new(&webcam)?;

    // let mut input_stream = EventStream::new();
    // let mut interval = tokio::time::interval(FRAME_DURATION);
    // let mut out = stdout();

    // out.queue(crossterm::cursor::Hide)?;
    // terminal::enable_raw_mode()?;
    // loop {
    //     tokio::select! {
    //         // TODO: refactor
    //         event = input_stream.next().fuse() => {
    //             match event {
    //                 Some(Ok(Event::Key(event))) => {
    //                     match keyboard.input(&event)? {
    //                         InputType::Exit => break,
    //                         input => {
    //                             ui.update(input)?;
    //                         },
    //                     }
    //                 },
    //                 _ => continue,
    //             }
    //         },
    //         _ = interval.tick() => {
    //             ui.update(InputType::Webcam { webcam: &mut webcam })?;
    //             ui.print(&mut out)?
    //         }
    //     }
    // }
    // out.queue(crossterm::cursor::Show)?;
    // terminal::disable_raw_mode()?;
    
    let client = Client::new();
    let response = create_room(&client).await?;
    println!("Created room: {}", response.room);

    println!("Received token: {}", response.token);
    connect_to_room(&response.token).await?;

    Ok(())
}
