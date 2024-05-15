// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::from_str;
use shared::{AuthMessage, CommandMessage, Message, TextMessage};
use std::time::SystemTime;
use std::vec;
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tracing::info;


struct TauriState {
    async_sender: mpsc::Sender<Message>,
    writehalf: Mutex<Option<OwnedWriteHalf>>,
    username: Mutex<Option<String>>,
    auth_token: Mutex<Option<String>>,
}

fn main() {
    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel::<Message>(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel::<Message>(1);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            send_message,
            request_channels,
            request_users,
            request_history,
            connect_server
            ])
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
                    match output {
                        Message::Text(_) => {
                            recieve_message(output, &app_handle);
                        },
                        Message::File(_) => todo!(),
                        Message::Command(_) => todo!(),
                        Message::Auth(_) => todo!(),
                        Message::Info(inner) => {
                            match inner.header.as_str() {
                                "channels" => init_channels(from_str(&inner.data).unwrap(), &app_handle),
                                "users" => init_users(from_str(&inner.data).unwrap(), &app_handle),
                                _ => {}
                            }
                            
                        },
                    }
                    
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn async_process_model(
    mut input_rx: mpsc::Receiver<Message>,
    output_tx: mpsc::Sender<Message>,
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

fn recieve_message<R: tauri::Runtime>(message: Message, manager: &impl Manager<R>) {
    info!(?message, "recieve_message");
    manager.emit_all("recieve_message", message).unwrap();
}

fn init_channels<R: tauri::Runtime>(channels: Vec<String>, manager: &impl Manager<R>) {
    info!(?channels, "init_channels");
    manager.emit_all("init_channels", channels).unwrap();
}

fn init_users<R: tauri::Runtime>(users: Vec<String>, manager: &impl Manager<R>) {
    info!(?users, "init_users");
    manager.emit_all("init_users", users).unwrap();
}

#[tauri::command]
async fn send_message(
    message: String,
    mut target: String,
    visibility: String,
    state: tauri::State<'_, TauriState>,
) -> Result<(), String> {
    info!(?message, "send_message");

    let lock_u = state.username.lock();
    let username = match &mut *lock_u.await {
        Some(usr) => usr.clone(),
        _ => {return Err("unable to fetch username: mutex poisoned".to_string());}
    };
    if visibility == "dm" {
        let mut users = vec![username.clone(), target.clone()];
        users.sort();
        target = format!("DM_{user1}_{user2}", user1 = users[0], user2 = users[1]);
    }
    let lock_a = state.auth_token.lock();
    let auth_token = match &mut *lock_a.await {
        Some(token) => token.clone(),
        _ => {return Err("unable to fetch auth-token: mutex poisoned".to_string());}
    };

    let msg_struct = Message::Text(TextMessage { 
        username: username, 
        auth_token: auth_token, 
        body: message, 
        channel: target, 
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
async fn request_channels(state: tauri::State<'_, TauriState>) -> Result<(), String> {
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

    let cmd_struct = Message::Command(CommandMessage {
        username, 
        auth_token, 
        command_type: String::from("channels"), 
        args: vec![] 
    });
    let formated_cmd_struct = serde_json::to_string(&cmd_struct).unwrap();
    let finalized_cmd: String = format!("{}\n", formated_cmd_struct);
    let byte_slice: &[u8] = finalized_cmd.as_bytes();

    let lock = state.writehalf.lock();
    match &mut *lock.await {
        Some(write_half) => {
            let _resulting = write_half.try_write(byte_slice).unwrap();
            println!("sent cmd requesting channel");
        }
        _ => {
            return Err(String::from("cmd unable to send: ownedwritehalf not initialised"));
        }
    }

    Ok(())
}

#[tauri::command]
async fn request_users(state: tauri::State<'_, TauriState>) -> Result<(), String> {
    let username = match &*state.username.lock().await {
        Some(inner) => inner.clone(),
        None => Err("Missing username")?
    };
    let auth_token = match &*state.auth_token.lock().await {
        Some(inner) => inner.clone(),
        None => Err("Missing auth token")?
    };
    let outgoing = serde_json::to_string(&Message::Command(CommandMessage {
        username,
        auth_token,
        command_type: "users".to_owned(),
        args: Vec::new()
    })).map_err(|err| err.to_string())? + "\n";
    match &mut *state.writehalf.lock().await {
        Some(stream) => stream.write_all(outgoing.as_bytes()).await.map_err(|err| err.to_string())?,
        None => Err("Missing writehalf")?
    };
    Ok(())
}

#[tauri::command]
async fn request_history(mut target: String, amount: String, visibility: String, state: tauri::State<'_, TauriState>) -> Result<(), String> {
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
    if visibility == "dm" {
        let mut users = vec![username.clone(), target.clone()];
        users.sort();
        target = format!("DM_{user1}_{user2}", user1 = users[0], user2 = users[1]);
    }
    let cmd_struct = Message::Command(CommandMessage { 
        username, 
        auth_token, 
        command_type: String::from("history"), 
        args: vec![target.clone(), amount]
    });
    let formated_cmd_struct = serde_json::to_string(&cmd_struct).unwrap();
    let finalized_cmd: String = format!("{}\n", formated_cmd_struct);
    let byte_slice: &[u8] = finalized_cmd.as_bytes();

    let lock = state.writehalf.lock();
    match &mut *lock.await {
        Some(write_half) => {
            let _resulting = write_half.try_write(byte_slice).unwrap();
            println!("sent cmd requesting history for channel {}", target);
        }
        _ => {
            return Err(String::from("cmd unable to send: ownedwritehalf not initialised"));
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
    let read_buff = BufReader::new(connection_read);
    tokio::spawn(connection_handler(read_buff, state_clone));

    return Ok(());
}

async fn connection_handler(
    mut read_buf: BufReader<OwnedReadHalf>,
    state: tauri::async_runtime::Sender<Message>,
) -> Result<(), String> {
    let mut buffer = String::new();

    loop {
        buffer.clear();

        if read_buf.read_line(&mut buffer).await.is_err() {
            return Err("end of stream".to_string());
        }

        if buffer == "" {
            return Err("end of stream".to_string());
        }

        let message = match from_str::<Message>(&buffer) {
            Ok(data) => data,
            Err(_) => return Err("error deserializing message".to_string()),
        };

        state.send(message.clone()).await.unwrap();
    }
}
