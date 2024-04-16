// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tauri::Manager;
use serde::{Serialize, Deserialize};
use tracing::info;

#[derive(Clone, Serialize)]
struct Payload {
  message: String,
}

struct AsyncProcInputTx {
  inner: Mutex<mpsc::Sender<String>>,
}

fn main() {
  let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel::<String>(1);
  let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel::<String>(1);

  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![greet, pri, send_message])
      .manage(AsyncProcInputTx {inner: Mutex::new(async_proc_input_tx)})
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
                  recv_message(output, &app_handle)
              }
          });

          Ok(())
      })
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}


async fn async_process_model(mut input_rx: mpsc::Receiver<String>, output_tx: mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  loop {
      while let Some(input) = input_rx.recv().await {
          let output = input;
          output_tx.send(output).await?;
      }
  }
}

fn recv_message<R: tauri::Runtime>(message: String, manager: &impl Manager<R>) {
  info!(?message, "recv_message");
  manager.emit_all("recv_message", message).unwrap();
}

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}!", name)
}

#[tauri::command]
fn pri(textt: &str) {
  println!("{}", textt)
}

#[tauri::command]
async fn send_message(message: String, state: tauri::State<'_, AsyncProcInputTx>) -> Result<(), String> {
  info!(?message, "send_message");
  let async_proc_input_tx = state.inner.lock().await;
  async_proc_input_tx.send(message).await.map_err(|e| e.to_string())
}