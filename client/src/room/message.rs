use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum RoomMessage {
    #[serde(rename = "chat")]
    Chat { user: String, content: String },

    #[serde(rename = "join")]
    Join { user: String },

    #[serde(rename = "leave")]
    Leave { user: String },
}
