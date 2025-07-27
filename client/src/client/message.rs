use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite;
use webrtc::{
    ice_transport::ice_candidate::RTCIceCandidate,
    peer_connection::sdp::session_description::RTCSessionDescription,
};

use crate::client::{Client, error::Error};

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

impl Stream for Client {
    type Item = Result<Message, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = match self.room_stream.as_mut() {
            Some(stream) => stream,
            None => return Poll::Ready(None),
        };

        match Pin::new(stream).poll_next(cx) {
            Poll::Ready(Some(Ok(message))) => match message {
                tungstenite::Message::Text(utf8_bytes) => {
                    match serde_json::from_str::<Message>(&utf8_bytes) {
                        Ok(deserialized) => Poll::Ready(Some(Ok(deserialized))),
                        Err(_) => Poll::Ready(Some(Err(Error::Deserialization))),
                    }
                }
                _ => Poll::Ready(None),
            },
            Poll::Ready(Some(Err(_))) => Poll::Ready(Some(Err(Error::ReceiveMessage))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
