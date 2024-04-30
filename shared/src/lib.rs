pub use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    Text(TextMessage),
    File(FileMessage),
    Command(CommandMessage),
    Auth(AuthMessage),
    Info(InfoMessage),
}

impl Message {
    pub fn username(&self) -> String {
        use Message as m;
        match self {
            m::Text(inner) => inner.username.clone(),
            m::File(inner) => inner.username.clone(),
            m::Command(inner) => inner.username.clone(),
            m::Auth(inner) => inner.username.clone(), 
            m::Info(_) => "Server".to_string()
        }
    }

    pub fn auth_token(&self) -> Option<String> {
        use Message as m;
        match self {
            m::Text(inner) => Some(inner.auth_token.clone()),
            m::File(inner) => Some(inner.auth_token.clone()),
            m::Command(inner) => Some(inner.auth_token.clone()),
            m::Auth(inner) => inner.auth_token.clone(),
            m::Info(_) => None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextMessage {
    pub username: String,
    pub auth_token: String,
    pub body: String,
    pub channel: String, // The message's target channel
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
    pub password: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InfoMessage {
    pub header: String,
    pub data: String
}
