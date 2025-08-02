#[derive(Debug)]
pub enum Error {
    OpenH264 { error: openh264::Error },
    Decoding,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::OpenH264 { error } => write!(f, "{}", error),
            Error::Decoding => write!(f, "nothing to decode"),
        }
    }
}

impl std::error::Error for Error {}
