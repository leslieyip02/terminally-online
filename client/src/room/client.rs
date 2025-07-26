use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{SinkExt, Stream};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite;

use crate::room::{
    RoomStream, connect_to_room, create_room, error::Error, join_room, message::RoomMessage,
};

pub struct RoomClient {
    http_client: reqwest::Client,
    room_stream: Option<RoomStream>,
}

impl RoomClient {
    const TIMEOUT: Duration = Duration::from_millis(3000);

    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            room_stream: None,
        }
    }

    pub async fn create_and_connect_to_room(&mut self) -> Result<String, Error> {
        let response = match timeout(Self::TIMEOUT, create_room(&self.http_client)).await {
            Ok(response) => response,
            Err(_) => return Err(Error::Timeout),
        };

        let (room, token) = match response {
            Ok(response) => (response.room, response.token),
            Err(_) => return Err(Error::CreateRoom),
        };

        self.room_stream = match self.connect_to_room(&token).await {
            Ok(stream) => Some(stream),
            Err(e) => return Err(e),
        };

        Ok(room)
    }

    pub async fn join_and_connect_to_room(&mut self, room_id: &str) -> Result<(), Error> {
        let response = match timeout(Self::TIMEOUT, join_room(&self.http_client, room_id)).await {
            Ok(response) => response,
            Err(_) => return Err(Error::Timeout),
        };

        let token = match response {
            Ok(response) => response.token,
            Err(_) => return Err(Error::JoinRoom { room_id: String::from(room_id) }),
        };

        self.room_stream = match self.connect_to_room(&token).await {
            Ok(stream) => Some(stream),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    pub async fn connect_to_room(&self, token: &str) -> Result<RoomStream, Error> {
        let response = match timeout(Self::TIMEOUT, connect_to_room(token)).await {
            Ok(response) => response,
            Err(_) => return Err(Error::Timeout),
        };

        match response {
            Ok(stream) => Ok(stream),
            Err(_) => Err(Error::WebSocket),
        }
    }

    pub async fn send_chat_message(&mut self, content: &str) -> Result<(), Error> {
        let stream = match &mut self.room_stream {
            Some(stream) => stream,
            None => return Err(Error::NotInRoom),
        };

        let message = RoomMessage::Chat {
            user: String::from("TODO"),
            content: String::from(content),
        };
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        let response = match timeout(Self::TIMEOUT, stream.send(json.into())).await {
            Ok(response) => response,
            Err(_) => return Err(Error::Timeout),
        };

        match response {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::SendMessage),
        }
    }
}

impl Stream for RoomClient {
    type Item = Result<RoomMessage, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = match self.room_stream.as_mut() {
            Some(stream) => stream,
            None => return Poll::Ready(None),
        };

        match Pin::new(stream).poll_next(cx) {
            Poll::Ready(Some(Ok(message))) => match message {
                tungstenite::Message::Text(utf8_bytes) => {
                    match serde_json::from_str::<RoomMessage>(&utf8_bytes) {
                        Ok(deserialized) => Poll::Ready(Some(Ok(deserialized))),
                        Err(_) => Poll::Ready(Some(Err(Error::Deserialization))),
                    }
                }
                _ => Poll::Ready(None),
            },
            Poll::Ready(Some(Err(_))) => Poll::Ready(None),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
