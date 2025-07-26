#[derive(Debug, Clone)]
pub enum Error {
    CreateRoom,
    JoinRoom { room_id: String },
    WebSocket,
    NotInRoom,
    Deserialization,
    Serialization,
    SendMessage,
    Timeout,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::CreateRoom => write!(f, "unable to create room"),
            Error::JoinRoom { room_id } => write!(f, "unable to join room {}", room_id),
            Error::WebSocket => write!(f, "unable to connect to web socket"),
            Error::NotInRoom => write!(f, "currently not connected to a room"),
            Error::Deserialization => write!(f, "unable to deserialize message"),
            Error::Serialization => write!(f, "unable to serialize message"),
            Error::SendMessage => write!(f, "unable to send message"),
            Error::Timeout => write!(f, "request timed out"),
        }
    }
}
