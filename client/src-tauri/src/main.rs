// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use shared::{Command, File, Message};
use std::time::SystemTime;
use tauri::Manager;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tracing::info;


struct TauriState {
    async_sender: mpsc::Sender<String>,
    writehalf: Mutex<Option<OwnedWriteHalf>>,
}

fn main() {
    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel::<String>(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel::<String>(1);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![send_message, connect_server])
        .manage(TauriState {
            async_sender: async_proc_input_tx,
            writehalf: None.into(),
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
    info!(?message, "recv_message");
    manager.emit_all("recv_message", message).unwrap();
}

#[tauri::command]
async fn send_message(
    message: String,
    state: tauri::State<'_, TauriState>,
) -> Result<(), String> {
    info!(?message, "send_message");

    let msg_struct = Message {
        username: String::from("Hello"),
        auth_token: String::from("hello"),
        body: String::from(message.clone()),
        embed_pointer: None,
        embed_type: None,
        message_id: None,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
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
async fn connect_server(ip: &str, state: tauri::State<'_, TauriState>) -> Result<(), String> {
    let stream = match TcpStream::connect(ip).await {
        Ok(stream) => stream,
        Err(e) => {
            println!("{}", format!("{}", e));
            return Err(e.to_string());
        }
    };
    println!("connected to {}", ip);

    let (connection_read, connection_write) = stream.into_split();

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
                Ok(msg) => {
                    let _ = state.send(msg.body).await;
                }
                Err(err) => {
                    return Err(format!("Error deserializing message: {}", err));
                }
            }
        }
    }
}
