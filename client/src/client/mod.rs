use std::{sync::Arc, time::Duration};

use futures::SinkExt;
use tokio::time::timeout;
use webrtc::peer_connection::{RTCPeerConnection, sdp::session_description::RTCSessionDescription};

use crate::{
    chat::command::{ChatboxCommand, ChatboxInput},
    client::{
        error::Error,
        message::{Message, SignalMessage},
        room::RoomStream,
        signaling::create_peer_connction,
    },
};

pub mod error;
pub mod message;
pub mod room;
pub mod signaling;

pub struct Client {
    username: String,
    http_client: reqwest::Client,
    room_stream: Option<RoomStream>,
    peer_connection: Option<Arc<RTCPeerConnection>>,
}

impl Client {
    const TIMEOUT: Duration = Duration::from_millis(3000);

    pub fn new() -> Self {
        let username = whoami::username();

        Self {
            username: username,
            http_client: reqwest::Client::new(),
            room_stream: None,
            peer_connection: None,
        }
    }

    pub async fn init(&mut self) -> Result<(), Error> {
        self.peer_connection = match create_peer_connction().await {
            Ok(peer_connection) => Some(Arc::new(peer_connection)),
            Err(e) => return Err(Error::WebRTC { error: e }),
        };

        Ok(())
    }

    async fn send_message(&mut self, json: String) -> Result<(), Error> {
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

    pub async fn receive_input(&mut self, input: &ChatboxInput) -> Result<Option<String>, Error> {
        match input {
            ChatboxInput::Message(content) => {
                self.send_chat_message(content).await?;
                Ok(None)
            }
            ChatboxInput::Command(command) => match command {
                ChatboxCommand::Create => {
                    let room_id = self.create_and_connect_to_room().await?;
                    Ok(format!("room id = {}", room_id).into())
                }
                ChatboxCommand::Join { room_id } => {
                    self.join_and_connect_to_room(room_id).await?;
                    Ok(None)
                }
                ChatboxCommand::Stream => {
                    self.send_offer_message().await?;
                    Ok(None)
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    pub async fn receive_message(&mut self, message: &Message) -> Result<(), Error> {
        match message {
            Message::Room { .. } => Ok(()),
            Message::Signal { signal_message } => {
                let peer_connection = match &mut self.peer_connection {
                    Some(peer_connection) => peer_connection,
                    None => return Err(Error::PeerConnectionNotReady),
                };

                match signal_message {
                    SignalMessage::Offer { payload } => self.handle_offer_message(&payload).await,
                    SignalMessage::Answer { payload } => Ok(()),
                    SignalMessage::Candidate { payload } => Ok(()),
                }
            }
        }
    }
}

pub(crate) trait RoomHandler {
    async fn create_and_connect_to_room(&mut self) -> Result<String, Error>;
    async fn join_and_connect_to_room(&mut self, room_id: &str) -> Result<(), Error>;
    async fn connect_to_room(&self, token: &str) -> Result<RoomStream, Error>;
    async fn send_chat_message(&mut self, content: &str) -> Result<(), Error>;
}

pub(crate) trait SignalHandler {
    async fn send_offer_message(&mut self) -> Result<(), Error>;
    async fn handle_offer_message(&mut self, offer: &RTCSessionDescription) -> Result<(), Error>;
    async fn handle_answer_message() -> Result<(), Error>;
}
