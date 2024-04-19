use base64ct::{Base64, Encoding};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use rusqlite::{Connection, OptionalExtension};
use shared::{AuthMessage, CommandMessage, Message, TextMessage};
use serde_json::{from_str, to_string};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, tcp};
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use toml::Table;

struct Server<> {
    config: Table,
    write_streams: Arc<Mutex<HashMap<std::net::SocketAddr, tcp::OwnedWriteHalf>>>,
    database: Arc<Mutex<Connection>>,
    session_tokens: Arc<Mutex<HashMap<String, String>>>,
}

impl Server {
    fn new(config: String) -> Result<Server, Box<dyn Error>>{
        let config = config.parse::<Table>()?;
        let db_address = config["db_path"].as_str().ok_or("Invalid database path")?;

        // Make sure that the password hash and salt tables exist
        let connection = Connection::open(db_address)?;
        connection.execute(
            "
            CREATE TABLE IF NOT EXISTS hashes
            (
                username TEXT PRIMARY KEY,
                hash TEXT NOT NULL
            );
            ",
            ()
        )?;
        connection.execute(
            "
            CREATE TABLE IF NOT EXISTS salts
            (
                username TEXT PRIMARY KEY,
                salt TEXT NOT NULL
            );
            ",
            ()
        )?;

        let database = Arc::new(Mutex::new(connection));
        Ok(Server {
            config,
            write_streams: Arc::new(Mutex::new(HashMap::new())),
            database,
            session_tokens: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn start(mut self) -> Result<(), Box<dyn Error>>{
        let listener = TcpListener::bind(self.config["listen_address"].as_str().ok_or("Invalid address")?).await?;

        let (text_tx_raw, text_rx_raw) = mpsc::channel::<TextMessage>(1);
        let (text_tx_processed, text_rx_processed) = mpsc::channel::<TextMessage>(1);
        // The command channel needs the address of the caller for sending targeted data instead of broadcasting
        let (command_tx, command_rx) = mpsc::channel::<(CommandMessage, SocketAddr)>(1);

        self.begin_listen(listener, text_tx_raw, command_tx);
        self.begin_broadcast(text_rx_processed);
        tokio::spawn(process_and_save(self.database.clone(),text_rx_raw,text_tx_processed));
        self.begin_parse_commands(command_rx);
        
        Ok(())
    }

    fn begin_listen(&mut self, listener: TcpListener, text_tx: mpsc::Sender<TextMessage>, command_tx: mpsc::Sender<(CommandMessage, SocketAddr)>) {
        let write_streams = self.write_streams.clone();
        let db = self.database.clone();
        let tokens = self.session_tokens.clone();
        tokio::spawn(async move {
            loop {
                let (stream, address) = match listener.accept().await {
                    Ok(val) => val, 
                    Err(_) => {
                        eprintln!("Failed to accept incoming TCP stream");
                        continue;
                    }
                };
                let (read_stream, mut write_stream) = stream.into_split();
                let mut reader = BufReader::new(read_stream);
                match authenticate(&mut reader, db.clone(), tokens.clone()).await {
                    Ok(Some(response)) => {
                        match write_stream.write_all(response.as_bytes()).await {
                            Ok(_) => {},
                            Err(e) => eprintln!("{:?}", e)
                        }
                    },
                    Ok(None) => {continue},
                    Err(e) => {eprintln!("{}", e);}
                };
                
                let mut writes_locked = write_streams.lock().await;
                writes_locked.insert(address, write_stream);
                tokio::spawn(listen_messages(reader, address, tokens.clone(), text_tx.clone(), command_tx.clone()));
            }
        });
    }

    fn begin_broadcast(&self, mut message_rx: mpsc::Receiver<TextMessage>) {
        let write_streams = self.write_streams.clone();
        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                print!("<{}> {}", message.username, message.body);
                let message = Message::Text(message);
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
                streams_locked.retain(|addr, _stream| !bad_addresses.contains(&addr));
            }
        });
    }

    /// Start a task to parse commands and send them off to be handled
    fn begin_parse_commands(&self, mut command_rx: mpsc::Receiver<(CommandMessage, SocketAddr)>) {
        let write_streams = self.write_streams.clone();
        let db = self.database.clone();
        tokio::spawn(async move {
            while let Some((cmd, address)) = command_rx.recv().await {
                match cmd.command_type.as_str() {
                    // User requests message history
                    "history" => {
                        let (_channel, count) = (cmd.args[0].clone(), cmd.args[1].clone());
                        let count = match count.parse() {
                            Ok(res) => res,
                            Err(_) => continue // We will not crashing the thread over a malformed command thank you very much
                        };
                        match retrieve_history(address, db.clone(), count, write_streams.clone()).await {
                            Ok(_) => {},
                            Err(error) => eprintln!("{:?}", error)
                        }
                    }
                    _ => {}
                }
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

async fn listen_messages(
    mut reader: BufReader<tcp::OwnedReadHalf>,
    address: SocketAddr,
    tokens: Arc<Mutex<HashMap<String, String>>>,
    text_tx: mpsc::Sender<TextMessage>,
    command_tx: mpsc::Sender<(CommandMessage, SocketAddr)>,
) {
    use shared::Message as m;
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
        if !is_authentic(message.clone(), tokens.clone()).await {
            eprintln!("User {} provided an invalid session token, disconnecting...", message.username());
            return
        }
        match message {
            m::Text(content) => {text_tx.send(content).await.unwrap();}, // Send off to save in database and broadcast
            m::Command(content) => {command_tx.send((content, address)).await.unwrap();},
            _ => {}
        };
    }
}

/// Add server-supplied data and write chat history to the server's database
async fn process_and_save(
    db: Arc<Mutex<Connection>>,
    mut text_rx_raw: mpsc::Receiver<TextMessage>,
    text_tx_processed: mpsc::Sender<TextMessage>,
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

/// Retrieve a specified number of messages from the history database and send them to the user who requested them
async fn retrieve_history(
    address: std::net::SocketAddr,
    db: Arc<Mutex<Connection>>,
    count: usize,
    streams: Arc<Mutex<HashMap<std::net::SocketAddr, tcp::OwnedWriteHalf>>>
) -> rusqlite::Result<()> {
    // Get the messages from the database
    let messages = {
        let db_locked = db.lock().await;
        let mut statement = db_locked.prepare(&format!("SELECT * FROM history ORDER BY id DESC LIMIT {count};"))?;
        let mut msgs = statement.query_map((), |row| {
            // The query_map *has* to return an iterator of rusql::Result
            Ok(Message::Text(TextMessage {
                message_id: row.get(0)?,
                username: row.get(1)?,
                body: row.get(2)?,
                timestamp: row.get(3)?,
                embed_pointer: row.get::<usize, Option<usize>>(4)?,
                embed_type: row.get::<usize, Option<String>>(5)?,
                auth_token: "".to_owned()
            }))
        })?
            .map(|res| res.unwrap())
            .collect::<Vec<Message>>();
        msgs.reverse(); // Rows come in the wrong order by default
        msgs.clone()
    };
    // Write the messages
    let mut streams_locked = streams.lock().await;
    let stream = streams_locked.get_mut(&address).unwrap();
    for message in messages {
        let serialized = to_string(&message).unwrap() + "\n";
        let _res = stream.write_all(&serialized.as_bytes()).await;
    }
    Ok(())
}

async fn authenticate(
    reader: &mut BufReader<tcp::OwnedReadHalf>,
    db: Arc<Mutex<Connection>>,
    session_tokens: Arc<Mutex<HashMap<String, String>>>
) -> Result<Option<String>, String> {
    use shared::Message as m;
    let mut buf = String::new();
    reader.read_line(&mut buf).await.map_err(|e| e.to_string())?;
    let auth_msg = from_str::<Message>(&buf).map_err(|e| e.to_string())?;
    let auth_msg = match auth_msg {
        m::Auth(inner) => inner,
        _ => return Err("Did not receive authentication message".to_owned())
    };
    let auth_successful = {
        // Get or generate the salt
        let db_locked = db.lock().await;
        let mut statement = db_locked
            .prepare(&format!("SELECT salt FROM salts WHERE username='{}';",auth_msg.username))
            .map_err(|e| e.to_string())?;
        let salt_option = statement
            .query_row((), |row| row.get::<usize, String>(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let salt = match salt_option {
            Some(salt) => salt,
            None => {
                // If there is no salt, generate one and insert it into the database
                let salt = rand_string(50);
                db_locked.execute(
                    &format!("INSERT INTO salts (username, salt) VALUES('{}', '{}');", auth_msg.username, salt),
                    ()
                ).map_err(|e| e.to_string())?;
                salt
            }
        };

        // Salt and hash password
        let mut hasher = Sha256::new();
        hasher.update((auth_msg.password.unwrap() + &salt).as_bytes());
        let hashed_pass = hasher.finalize();
        let hashed_pass = Base64::encode_string(&hashed_pass);

        // Check or update the hashed password against the database
        let mut statement = db_locked
            .prepare(&format!("SELECT hash FROM hashes WHERE username = '{}';", auth_msg.username))
            .map_err(|e| e.to_string())?;
        let hash_option = statement
            .query_row((), |row| row.get::<usize, String>(0))
            .optional()
            .map_err(|e| e.to_string())?;
        match hash_option {
            Some(hash) => hash == hashed_pass,
            None => {
                db_locked.execute(
                    &format!("INSERT INTO hashes (username, hash) VALUES('{}', '{}');", auth_msg.username, hashed_pass),
                    ()
                ).map_err(|e| e.to_string())?;
                true
            }
        }
    };
    if auth_successful {
        let mut tokens_locked = session_tokens.lock().await;
        let token = rand_string(100);
        tokens_locked.insert(auth_msg.username.clone(), token.clone());
        let response = Message::Auth(AuthMessage {
            username: auth_msg.username,
            auth_token: Some(token),
            password: None
        });
        return Ok(Some(to_string(&response).map_err(|e| e.to_string())? + "\n"))
    }
    Ok(None)
}

fn rand_string(len: usize) -> String {
    thread_rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(|byte| char::from(byte))
        .collect::<String>()
}

async fn is_authentic(message: Message, tokens: Arc<Mutex<HashMap<String, String>>>) -> bool {
    let username = message.username();
    let tokens_locked = tokens.lock().await;
    let stored_token = tokens_locked[&username].clone();
    let message_token = match message.auth_token() {
        Some(inner) => inner,
        None => return false
    };
    return stored_token == message_token
}