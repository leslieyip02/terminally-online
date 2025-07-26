use serde::{Deserialize, Serialize};

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
