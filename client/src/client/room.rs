use futures::{StreamExt, stream::SplitSink};
use serde::Deserialize;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver},
    time::timeout,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{self, client::IntoClientRequest},
};

use crate::client::{
    Client, RoomHandler,
    error::Error,
    message::{Message, RoomMessage, convert_stream_message},
};

pub type WriteStream = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>;
pub type MessageReceiver = Receiver<Result<Message, Error>>;

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
) -> Result<(WriteStream, MessageReceiver), Box<dyn std::error::Error>> {
    let url = format!(
        "ws://localhost:8080/ws?token={}&username={}",
        token, username
    )
    .into_client_request()?;

    let (stream, _) = connect_async(url).await?;
    let (write_stream, mut read_stream) = stream.split();

    let (tx, rx) = mpsc::channel(32);
    tokio::spawn(async move {
        while let Some(Ok(stream_message)) = read_stream.next().await {
            let message = convert_stream_message(&stream_message);
            match tx.send(message).await {
                Ok(()) => {}
                Err(_) => break,
            }
        }
    });

    Ok((write_stream, rx))
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

        let (write_stream, rx) = self.connect_to_room(&token).await?;
        self.write_stream = Some(write_stream);
        self.message_receiver = Some(rx);

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

        let (write_stream, rx) = self.connect_to_room(&token).await?;
        self.write_stream = Some(write_stream);
        self.message_receiver = Some(rx);

        Ok(())
    }

    async fn connect_to_room(&self, token: &str) -> Result<(WriteStream, MessageReceiver), Error> {
        let response = match timeout(Self::TIMEOUT, connect_to_room(token, &self.username)).await {
            Ok(response) => response,
            Err(_) => return Err(Error::Timeout),
        };

        response.map_err(|_| Error::WebSocket)
    }

    async fn send_chat(&mut self, content: &str) -> Result<(), Error> {
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
