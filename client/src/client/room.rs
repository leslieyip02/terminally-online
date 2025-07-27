use serde::Deserialize;
use tokio::{net::TcpStream, time::timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::client::IntoClientRequest,
};

use crate::client::{error::Error, message::RoomMessage, Client, RoomHandler};

pub type RoomStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Deserialize)]
struct CreateRoomResponse {
    pub(crate) room: String,
    pub(crate) token: String,
}

async fn create_room(
    client: &reqwest::Client,
) -> Result<CreateRoomResponse, Box<dyn std::error::Error>> {
    let url = "http://localhost:8080/create";
    let response = client
        .post(url)
        .send()
        .await?
        .json::<CreateRoomResponse>()
        .await?;
    Ok(response)
}

#[derive(Deserialize)]
struct JoinRoomResponse {
    pub(crate) token: String,
}

async fn join_room(
    client: &reqwest::Client,
    room_id: &str,
) -> Result<JoinRoomResponse, Box<dyn std::error::Error>> {
    let url = format!("http://localhost:8080/join/{}", room_id);
    let response = client
        .post(&url)
        .send()
        .await?
        .json::<JoinRoomResponse>()
        .await?;
    Ok(response)
}

async fn connect_to_room(
    token: &str,
    username: &str,
) -> Result<RoomStream, Box<dyn std::error::Error>> {
    let url = format!(
        "ws://localhost:8080/ws?token={}&username={}",
        token, username
    )
    .into_client_request()?;
    let (stream, _) = connect_async(url).await?;
    Ok(stream)
}

impl RoomHandler for Client {
    async fn create_and_connect_to_room(&mut self) -> Result<String, Error> {
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

    async fn join_and_connect_to_room(&mut self, room_id: &str) -> Result<(), Error> {
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

    async fn connect_to_room(&self, token: &str) -> Result<RoomStream, Error> {
        let response = match timeout(Self::TIMEOUT, connect_to_room(token, &self.username)).await {
            Ok(response) => response,
            Err(_) => return Err(Error::Timeout),
        };

        match response {
            Ok(stream) => Ok(stream),
            Err(_) => Err(Error::WebSocket),
        }
    }

    async fn send_chat_message(&mut self, content: &str) -> Result<(), Error> {
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
}
