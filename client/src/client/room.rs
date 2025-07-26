use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::client::IntoClientRequest,
};

pub type RoomStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Deserialize)]
pub(crate) struct CreateRoomResponse {
    pub(crate) room: String,
    pub(crate) token: String,
}

pub(crate) async fn create_room(
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
pub(crate) struct JoinRoomResponse {
    pub(crate) token: String,
}

pub(crate) async fn join_room(
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

pub(crate) async fn connect_to_room(
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
