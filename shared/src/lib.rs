pub use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    Text(TextMessage),
    File(FileMessage),
    Command(CommandMessage),
    Auth(AuthMessage)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextMessage {
    pub username: String,
    pub auth_token: String,
    pub body: String,
    pub embed_pointer: Option<usize>,
    pub embed_type: Option<String>,
    pub message_id: Option<u32>,
    pub timestamp: u64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileMessage {
    pub username: String,
    pub auth_token: String,
    pub filename: String,
    pub data: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandMessage {
    pub username: String,
    pub auth_token: String,
    pub command_type: String,
    pub args: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthMessage {
    pub username: String,
    pub auth_token: Option<String>,
    pub pass_hash: Option<String>
}