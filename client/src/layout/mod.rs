use crate::{chat::Chatbox, video::VideoPanel};

pub fn create_layout() -> Result<(Chatbox, VideoPanel), Box<dyn std::error::Error>> {
    let size = match termsize::get() {
        Some(size) => size,
        None => panic!("Unable to get terminal size."),
    };

    let width = size.cols;
    let height = size.rows;

    // TODO: improve this
    if width < height {
        Ok((
            Chatbox::new(1, 34, width - 2, height - 34),
            VideoPanel::new(1, 1, width - 2, 32)?,
        ))
    } else {
        Ok((
            Chatbox::new(130, 1, width - 128 - 4, height - 2),
            VideoPanel::new(1, 1, 128, height - 2)?,
        ))
    }
}
