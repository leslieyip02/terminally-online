#[derive(Debug)]
pub enum Error {
    CreateRoom,
    JoinRoom { room_id: String },
    WebSocket,
    NotConnected,
    Deserialization,
    Serialization,
    SendMessage,
    ReceiveMessage,
    Timeout,
    AlreadyInitialized,
    PeerConnectionNotReady,
    WebcamNotReady,
    WebRTC { error: webrtc::Error },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::CreateRoom => write!(f, "unable to create room"),
            Error::JoinRoom { room_id } => write!(f, "unable to join room {}", room_id),
            Error::WebSocket => write!(f, "unable to connect to web socket"),
            Error::NotConnected => write!(f, "not connected to a room"),
            Error::Deserialization => write!(f, "unable to deserialize message"),
            Error::Serialization => write!(f, "unable to serialize message"),
            Error::SendMessage => write!(f, "unable to send message"),
            Error::ReceiveMessage => write!(f, "unable to receive message"),
            Error::Timeout => write!(f, "request timed out"),
            Error::AlreadyInitialized => write!(f, "init() has already been called"),
            Error::PeerConnectionNotReady => write!(f, "peer connection is not ready"),
            Error::WebcamNotReady => write!(f, "webcam is not ready"),
            Error::WebRTC { error } => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {}
