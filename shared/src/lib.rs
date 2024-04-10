use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub uuid: String,
    pub auth_token: String,
    pub body: String,
    pub embed_pointer: Option<usize>,
    pub embed_type: Option<String>,
    pub message_id: Option<u32>,
    pub timestamp: u64
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub uuid: String,
    pub auth_token: String,
    pub filename: String,
    pub data: String
}

#[derive(Serialize, Deserialize)]
pub struct Command {
    pub uuid: String,
    pub auth_token: String,
    pub command_type: String,
    pub args: Vec<String>
}