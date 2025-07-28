use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite;
use webrtc::{
    ice_transport::ice_candidate::RTCIceCandidate,
    peer_connection::sdp::session_description::RTCSessionDescription,
};

use crate::client::error::Error;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    Room {
        #[serde(flatten)]
        room_message: RoomMessage,
    },
    Signal {
        #[serde(flatten)]
        signal_message: SignalMessage,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum RoomMessage {
    #[serde(rename = "chat")]
    Chat { username: String, content: String },

    #[serde(rename = "join")]
    Join { username: String },

    #[serde(rename = "leave")]
    Leave { username: String },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SignalMessage {
    #[serde(rename = "offer")]
    Offer { payload: RTCSessionDescription },

    #[serde(rename = "answer")]
    Answer { payload: RTCSessionDescription },

    #[serde(rename = "candidate")]
    Candidate { payload: RTCIceCandidate },
}

pub(crate) fn convert_stream_message(message: &tungstenite::Message) -> Result<Message, Error> {
    let data = match message {
        tungstenite::Message::Text(data) => data,
        _ => return Err(Error::Deserialization),
    };

    serde_json::from_str::<Message>(&data).map_err(|_| Error::Deserialization)
}
