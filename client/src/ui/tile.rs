use crossterm::style::{self, StyledContent, Stylize};

const UPPER_HALF_BLOCK: char = '▀';
const ANSI_GRAYSCALE_BEGIN: u8 = 232;
const ANSI_RANGE: f32 = 256.0;

const ANSI_VALUES: [&str; 24] = [
    "232m", "233m", "234m", "235m", "236m", "237m", "238m", "239m", "240m", "241m", "242m", "243m",
    "244m", "245m", "246m", "247m", "248m", "249m", "250m", "251m", "252m", "253m", "254m", "255m",
];

#[derive(Clone, Copy)]
pub struct Tile {
    pub content: char,
    pub top_brightness: u8,
    pub bottom_brightness: u8,
}

impl Tile {
    pub fn reset(&mut self) {
        self.content = UPPER_HALF_BLOCK
    }

    pub fn styled(&self) -> StyledContent<char> {
        if self.content.is_ascii() {
            StyledContent::new(style::ContentStyle::default(), self.content)
        } else {
            let foreground_color = Tile::normalized(self.top_brightness);
            let background_color = Tile::normalized(self.bottom_brightness);
            self.content
                .with(style::Color::AnsiValue(foreground_color))
                .on(style::Color::AnsiValue(background_color))
        }
    }

    fn normalized(value: u8) -> u8 {
        ANSI_GRAYSCALE_BEGIN + ((value as f32) / ANSI_RANGE * ANSI_VALUES.len() as f32) as u8
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            content: UPPER_HALF_BLOCK,
            top_brightness: 0,
            bottom_brightness: 0,
        }
    }
}
