#[derive(Debug, Clone)]
pub enum Error {
    InvalidUsage { usage: String },
    InvalidCommand,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidUsage { usage } => write!(f, "usage: {}", usage),
            Error::InvalidCommand => write!(f, "invalid command"),
        }
    }
}
