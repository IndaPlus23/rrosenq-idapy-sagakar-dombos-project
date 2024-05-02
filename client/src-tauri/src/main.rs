// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use shared::{AuthMessage, Message, TextMessage};
use std::time::SystemTime;
use tauri::Manager;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tracing::info;


struct TauriState {
    async_sender: mpsc::Sender<String>,
    writehalf: Mutex<Option<OwnedWriteHalf>>,
    username: Mutex<Option<String>>,
    auth_token: Mutex<Option<String>>,
}

fn main() {
    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel::<String>(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel::<String>(1);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![send_message, connect_server])
        .manage(TauriState {
            async_sender: async_proc_input_tx,
            writehalf: None.into(),
            username: None.into(),
            auth_token: None.into(),
        })
        .setup(move |app| {
            let app_handle = app.handle();
            let async_proc_output_tx_clone = async_proc_output_tx.clone();
            tauri::async_runtime::spawn(async move {
                async_process_model(async_proc_input_rx, async_proc_output_tx_clone)
                    .await
                    .expect("error in async process model");
            });

            tauri::async_runtime::spawn(async move {
                while let Some(output) = async_proc_output_rx.recv().await {
                    recieve_message(output, &app_handle)
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn async_process_model(
    mut input_rx: mpsc::Receiver<String>,
    output_tx: mpsc::Sender<String>,
) -> Result<(), String> {
    while let Some(input) = input_rx.recv().await {
        let output = input;
        output_tx
            .send(output)
            .await
            .map_err(|e| e.to_string())
            .unwrap();
    }

    return Err(String::from("error in async-handler"));
}

fn recieve_message<R: tauri::Runtime>(message: String, manager: &impl Manager<R>) {
    info!(?message, "recieve_message");
    manager.emit_all("recieve_message", message).unwrap();
}

#[tauri::command]
async fn send_message(
    message: String,
    state: tauri::State<'_, TauriState>,
) -> Result<(), String> {
    info!(?message, "send_message");

    let lock_u = state.username.lock();
    let username = match &mut *lock_u.await {
        Some(usr) => usr.clone(),
        _ => {return Err("unable to fetch username: mutex poisoned".to_string());}
    };
    let lock_a = state.auth_token.lock();
    let auth_token = match &mut *lock_a.await {
        Some(token) => token.clone(),
        _ => {return Err("unable to fetch auth-token: mutex poisoned".to_string());}
    };

    let msg_struct = Message::Text(TextMessage { 
        username: username, 
        auth_token: auth_token, 
        body: message, 
        channel: String::from("general"), 
        embed_pointer: None, 
        embed_type: None, 
        message_id: None, 
        timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
    });
    let formated_msg_struct = serde_json::to_string(&msg_struct).unwrap();
    let finalized_msg: String = format!("{}\n", formated_msg_struct);

    let byte_slice: &[u8] = finalized_msg.as_bytes();

    let lock = state.writehalf.lock();
    match &mut *lock.await {
        Some(write_half) => {
            let resulting = write_half.try_write(byte_slice).unwrap();
            println!("sent message with {} bytes", resulting);
        }
        _ => {
            return Err(String::from("message unable to send: ownedwritehalf not initialised"));
        }
    }

    Ok(())
}

#[tauri::command]
async fn connect_server(ip: &str, username: String, password: String, state: tauri::State<'_, TauriState>) -> Result<(), String> {
    let stream = match TcpStream::connect(ip).await {
        Ok(stream) => stream,
        Err(e) => {
            println!("{}", format!("{}", e));
            return Err(e.to_string());
        }
    };
    println!("connected to {}, attempting authentication", ip);

    let (mut connection_read, connection_write) = stream.into_split();

    let auth_message = Message::Auth(AuthMessage {
        username: username.clone(), 
        auth_token: None, 
        password: Some(password.clone()) 
    });
    let formated_msg_struct = serde_json::to_string(&auth_message).unwrap();
    let finalized_msg = format!("{}\n", formated_msg_struct);

    let byte_slice: &[u8] = finalized_msg.as_bytes();

    match connection_write.try_write(byte_slice) {
        Ok(r) => {println!("wrote {} bytes during auth, awaiting response", r)},
        Err(e) => {return Err(e.to_string());}
    }

    let mut temp_buffer = vec![0u8; 1024];
    let temp_read = connection_read.read(&mut temp_buffer).await.map_err(|e| e.to_string()).unwrap();

    if let Ok(msg) = String::from_utf8(temp_buffer[..temp_read].to_vec()) {
        println!("{}", msg);
        let fixed: Result<shared::Message, _> = serde_json::from_str(&msg).map_err(|e| e.to_string());
        match fixed {
            Ok(auth_struct) => {
                match auth_struct {
                    Message::Auth(inner) => {
                        if inner.auth_token == None {
                            return Err("incorrect username or password".to_string());
                        } else {
                            *state.username.lock().await = Some(username.clone());
                            *state.auth_token.lock().await = Some(inner.auth_token.unwrap().clone());
                        }
                    },
                    _ => {
                        return Err("auth message was not returned".to_string());
                    }
                }
            },
            _ => {
                return Err("error deserializing auth message".to_string());
            }
        }
    };

    *state.writehalf.lock().await = Some(connection_write);

    let state_clone = state.async_sender.clone();
    tokio::spawn(connection_handler(connection_read, state_clone));

    return Ok(());
}

async fn connection_handler(
    mut readhalf: OwnedReadHalf,
    state: tauri::async_runtime::Sender<std::string::String>,
) -> Result<(), String> {
    let mut buffer = vec![0u8; 1024];

    loop {
        let bytes_read = readhalf
            .read(&mut buffer)
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        if bytes_read == 0 {
            return Err(String::from("connection ended"));
        }

        if let Ok(message) = String::from_utf8(buffer[..bytes_read].to_vec()) {
            let fixed: Result<shared::Message, _> =
                serde_json::from_str(&message).map_err(|e| e.to_string());
            match fixed {
                Ok(msg_struct) => {
                    match msg_struct {
                        Message::Text(msg) => state.send(msg.body).await.unwrap(),
                        _ => (),
                    }
                }
                Err(err) => {
                    return Err(format!("error deserializing message: {}", err));
                }
            }
        }
    }
}