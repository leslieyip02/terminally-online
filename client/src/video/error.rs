#[derive(Debug)]
pub enum Error {
    CameraNotReady { error: nokhwa::NokhwaError },
    OpenH264 { error: openh264::Error },
    Decoding,
    MalformedNalUnit,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::CameraNotReady { error } => write!(f, "{}", error),
            Error::OpenH264 { error } => write!(f, "{}", error),
            Error::Decoding => write!(f, "nothing to decode"),
            Error::MalformedNalUnit => write!(f, "malformed NAL unit"),
        }
    }
}

impl std::error::Error for Error {}
