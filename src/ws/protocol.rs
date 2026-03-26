use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMsg {
    #[serde(rename = "dm")]
    Dm { to: String, body: String },
    #[serde(rename = "join_room")]
    JoinRoom { room: String },
    #[serde(rename = "room_msg")]
    RoomMsg { room: String, body: String },
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMsg {
    #[serde(rename = "dm")]
    Dm { from: String, body: String },
    #[serde(rename = "room_msg")]
    RoomMsg { room: String, from: String, body: String },
    #[serde(rename = "system")]
    System { message: String },
}

