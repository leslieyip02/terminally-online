use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::client::IntoClientRequest,
};

pub type RoomStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub mod client;
pub mod error;
pub mod message;

#[derive(Deserialize)]
struct CreateRoomResponse {
    room: String,
    token: String,
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
    token: String,
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

async fn connect_to_room(token: &str) -> Result<RoomStream, Box<dyn std::error::Error>> {
    let url = format!("ws://localhost:8080/ws?token={}", token).into_client_request()?;
    let (stream, _) = connect_async(url).await?;
    Ok(stream)
}
