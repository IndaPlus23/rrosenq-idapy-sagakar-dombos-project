use shared::Message;
use serde_json::{from_str, to_string};
use std::sync::Arc;
use tokio::net::{TcpListener, tcp};
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use toml::Table;

struct Server<> {
    config: Table,
    write_streams: Arc<Mutex<Vec<(std::net::SocketAddr, tcp::OwnedWriteHalf)>>>,
}

impl Server {
    fn new(config: String) -> Server{
        let config = config.parse::<Table>().expect("Failed to parse config file");
        Server {
            config,
            write_streams: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn start(mut self) -> Result<(), Box<dyn std::error::Error>>{
        let listener = TcpListener::bind(self.config["listen_address"].as_str().ok_or("Invalid address")?).await?;
        let (message_tx, message_rx) = mpsc::channel::<Message>(1);
        self.begin_listen(listener, message_tx);
        self.begin_broadcast(message_rx);
        Ok(())
    }

    fn begin_listen(&mut self, listener: TcpListener, message_tx: mpsc::Sender<Message>) {
        let write_streams = self.write_streams.clone();
        tokio::spawn(async move {
            loop {
                let (stream, address) = match listener.accept().await {
                    Ok(val) => val, 
                    Err(_) => {
                        eprintln!("Failed to accept incoming TCP stream");
                        continue;
                    }
                };
                let (read_stream, write_stream) = stream.into_split();
                let mut writes_locked = write_streams.lock().await;
                writes_locked.push((address, write_stream));
                tokio::spawn(listen_messages(read_stream, message_tx.clone()));
            
            }
        });
    }

    fn begin_broadcast(&self, mut message_rx: mpsc::Receiver<Message>) {
        let write_streams = self.write_streams.clone();
        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                print!("<{}> {}", message.username, message.body);
                let mut bad_addresses: Vec<std::net::SocketAddr> = Vec::new();
                let mut streams_locked = write_streams.lock().await;
                for (address, stream) in streams_locked.iter_mut() {
                    let outgoing_message = match to_string(&message) {
                        Ok(data) => data,
                        Err(_) => return,
                    };
                    let outgoing_message = outgoing_message + "\n";
                    match stream.write_all(outgoing_message.as_bytes()).await {
                        Ok(_) => {},
                        Err(_) => {
                            let _res = stream.shutdown().await;
                            bad_addresses.push(address.clone())
                        }
                    }
                }
                streams_locked.retain(|(addr, _stream)| !bad_addresses.contains(&addr));
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let config = std::fs::read_to_string("config.toml").expect("Failed to read config file");
    let server = Server::new(config);
    server.start().await.expect("Failed to start server");
    loop {}
}



async fn listen_messages(stream: tcp::OwnedReadHalf, message_tx: mpsc::Sender<Message>) {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();
    loop {
        buffer.clear();
        if reader.read_line(&mut buffer).await.is_err() {
            return; // If we receive some nonsense or the stream is dead, end the task
        }
        let message = match from_str::<Message>(&buffer) {
            Ok(data) => data,
            Err(_) => return 
        };
        message_tx.send(message).await.unwrap();
    }
}