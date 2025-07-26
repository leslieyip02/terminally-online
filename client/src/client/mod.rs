use std::time::Duration;

use futures::SinkExt;
use tokio::time::timeout;
use webrtc::{
    ice_transport::ice_candidate::RTCIceCandidateInit,
    peer_connection::sdp::session_description::RTCSessionDescription,
};

use crate::client::{
    error::Error,
    message::{RoomMessage, SignalMessage},
    room::{RoomStream, connect_to_room, create_room, join_room},
};

pub mod error;
pub mod message;
pub mod room;
pub mod signaling;

pub struct Client {
    username: String,
    http_client: reqwest::Client,
    room_stream: Option<RoomStream>,
}

impl Client {
    const TIMEOUT: Duration = Duration::from_millis(3000);

    pub fn new() -> Self {
        let username = whoami::username();

        Self {
            username: username,
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
            Err(_) => {
                return Err(Error::JoinRoom {
                    room_id: String::from(room_id),
                });
            }
        };

        self.room_stream = match self.connect_to_room(&token).await {
            Ok(stream) => Some(stream),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    pub async fn connect_to_room(&self, token: &str) -> Result<RoomStream, Error> {
        let response = match timeout(Self::TIMEOUT, connect_to_room(token, &self.username)).await {
            Ok(response) => response,
            Err(_) => return Err(Error::Timeout),
        };

        match response {
            Ok(stream) => Ok(stream),
            Err(_) => Err(Error::WebSocket),
        }
    }

    pub async fn send_chat_message(&mut self, content: &str) -> Result<(), Error> {
        let message = RoomMessage::Chat {
            username: self.username.clone(),
            content: String::from(content),
        };
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        self.send_message(json).await
    }

    pub(crate) async fn send_message(&mut self, json: String) -> Result<(), Error> {
        let stream = match &mut self.room_stream {
            Some(stream) => stream,
            None => return Err(Error::NotConnected),
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

    // TODO: these can probably be removed

    pub async fn send_offer_message(
        &mut self,
        payload: &RTCSessionDescription,
    ) -> Result<(), Error> {
        let message = SignalMessage::Offer {
            payload: payload.clone(),
        };
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        self.send_message(json).await
    }

    pub async fn send_answer_message(
        &mut self,
        payload: &RTCSessionDescription,
    ) -> Result<(), Error> {
        let message = SignalMessage::Answer {
            payload: payload.clone(),
        };
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        self.send_message(json).await
    }

    pub async fn send_candidate_message(
        &mut self,
        payload: &RTCIceCandidateInit,
    ) -> Result<(), Error> {
        let message = SignalMessage::Candidate {
            payload: payload.clone(),
        };
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(_) => return Err(Error::Serialization),
        };

        self.send_message(json).await
    }
}
