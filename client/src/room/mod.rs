use std::time::Duration;

use futures::SinkExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};

const REQUEST_TIMEOUT: Duration = Duration::from_millis(3000);

#[derive(Deserialize)]
pub struct CreateRoomResponse {
    pub room: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct JoinRoomResponse {
    pub token: String,
}

pub async fn create_room_with_timeout(
    client: &Client,
) -> Result<CreateRoomResponse, Box<dyn std::error::Error>> {
    tokio::time::timeout(REQUEST_TIMEOUT, create_room(&client)).await?
}

async fn create_room(client: &Client) -> Result<CreateRoomResponse, Box<dyn std::error::Error>> {
    let url = "http://localhost:8080/create";
    let response = client
        .post(url)
        .send()
        .await?
        .json::<CreateRoomResponse>()
        .await?;
    Ok(response)
}

pub async fn join_room_with_timeout(
    client: &Client,
    room_id: &str,
) -> Result<JoinRoomResponse, Box<dyn std::error::Error>> {
    tokio::time::timeout(REQUEST_TIMEOUT, join_room(&client, &room_id)).await?
}

async fn join_room(
    client: &Client,
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

#[derive(Serialize)]
struct ChatMessage {
    r#type: String,
    content: String,
}

pub async fn connect_to_room(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("ws://localhost:8080/ws?token={}", token).into_client_request()?;
    let (mut ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connected!");

    let message = ChatMessage {
        r#type: String::from("chat"),
        content: String::from("abcd"),
    };
    let json = serde_json::to_string(&message)?;
    ws_stream.send(json.into()).await?;

    Ok(())
}
