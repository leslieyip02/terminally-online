use std::io::stdout;

use crossterm::{
    QueueableCommand,
    event::{Event, EventStream},
    terminal,
};

use client::{
    input::{InputHandler, InputType},
    ui::{FRAME_DURATION, Printable, Ui, Updatable},
};
use futures::{FutureExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ui = Ui::new()?;
    let mut input_handler = InputHandler::new();

    let mut input_stream = EventStream::new();
    let mut interval = tokio::time::interval(FRAME_DURATION);
    let mut out = stdout();

    out.queue(crossterm::cursor::Hide)?;
    terminal::enable_raw_mode()?;
    loop {
        tokio::select! {
            event = input_stream.next().fuse() => {
                match event {
                    Some(Ok(Event::Key(event))) => {
                        match input_handler.input(&event)? {
                            InputType::Exit => break,
                            input => {
                                ui.update(input)?;
                            },
                        }
                    },
                    _ => continue,
                }
            },
            _ = interval.tick() => {
                ui.update(InputType::TickUpdate)?;
                ui.print(&mut out)?
            }
        }
    }
    out.queue(crossterm::cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
