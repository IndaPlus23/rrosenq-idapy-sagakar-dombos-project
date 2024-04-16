use rusqlite::Connection;
use shared::{Message, TextMessage};
use serde_json::{from_str, to_string};
use std::error::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, tcp};
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use toml::Table;

struct Server<> {
    config: Table,
    write_streams: Arc<Mutex<Vec<(std::net::SocketAddr, tcp::OwnedWriteHalf)>>>,
    database: Arc<Mutex<Connection>>,
}

impl Server {
    fn new(config: String) -> Result<Server, Box<dyn Error>>{
        let config = config.parse::<Table>()?;
        let db_address = config["db_path"].as_str().ok_or("Invalid database path")?;
        let database = Arc::new(Mutex::new(Connection::open(db_address)?));
        Ok(Server {
            config,
            write_streams: Arc::new(Mutex::new(Vec::new())),
            database,
        })
    }

    async fn start(mut self) -> Result<(), Box<dyn Error>>{
        let listener = TcpListener::bind(self.config["listen_address"].as_str().ok_or("Invalid address")?).await?;

        let (text_tx_raw, text_rx_raw) = mpsc::channel::<TextMessage>(1);
        let (text_tx_processed, text_rx_processed) = mpsc::channel::<TextMessage>(1);

        self.begin_listen(listener, text_tx_raw);
        self.begin_broadcast(text_rx_processed);
        tokio::spawn(process_and_save(self.database.clone(), text_rx_raw, text_tx_processed));
        
        Ok(())
    }

    fn begin_listen(&mut self, listener: TcpListener, text_tx: mpsc::Sender<TextMessage>) {
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
                tokio::spawn(listen_messages(read_stream, text_tx.clone()));
            
            }
        });
    }

    fn begin_broadcast(&self, mut message_rx: mpsc::Receiver<TextMessage>) {
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
    let server = Server::new(config).expect("Failed to start server");
    server.start().await.expect("Failed to start server");
    loop {}
}

async fn listen_messages(stream: tcp::OwnedReadHalf, text_tx: mpsc::Sender<TextMessage>) {
    use shared::Message as m;
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
        match message {
            m::Text(content) => {text_tx.send(content).await.unwrap();},
            _ => {}
        };
    }
}

/// Add server-supplied data and write chat history to the server's database
async fn process_and_save(
    db: Arc<Mutex<Connection>>,
    mut text_rx_raw: mpsc::Receiver<TextMessage>,
    text_tx_processed: mpsc::Sender<TextMessage>
) -> rusqlite::Result<()> {
    // Make sure that we actually have a history table
    {
        let db_locked = db.lock().await;
        let query = "
        CREATE TABLE IF NOT EXISTS history
        (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            body TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            embed_pointer INTEGER,
            embed_type TEXT
        );";
        db_locked.execute(&query, ()).unwrap();
    }
    // Await messages to process and send
    while let Some(mut msg) = text_rx_raw.recv().await {
        // First process the message by adding server-supplied data
        msg.timestamp = match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(time) => time.as_secs(),
            Err(_) => 0,
        };
        
        
        // Then write it to the database
        // This scope is important in case of any awaits afterwards
        {
            // Write
            let query = format!("
            INSERT INTO history (username, body, timestamp)
            VALUES('{username}', '{body}', '{timestamp}');
            ", username = msg.username, body = msg.body, timestamp = msg.timestamp);
            let db_locked = db.lock().await;
            db_locked.execute(&query, ()).unwrap();

            // Retrieve id
            msg.message_id = Some(db_locked.last_insert_rowid() as u32);
        }

        // When we know the write is successful, send the processed message to be broadcast
        text_tx_processed.send(msg).await.unwrap();
    }
    println!("process and save exiting");
    Ok(())
}