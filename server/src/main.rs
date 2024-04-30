use base64ct::{Base64, Encoding};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use rusqlite::{Connection, OptionalExtension};
use shared::{AuthMessage, CommandMessage, InfoMessage, Message, TextMessage};
use serde_json::{from_str, to_string};
use sha2::{Sha256, Digest};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::net::{TcpListener, tcp};
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use toml::Table;

// Holds data and essential helper methods for the chat server
struct Server<> {
     // A Key:value table of config settings loaded from config.toml
    config: Table,
    // Holds socket addressess and their corresponding write streams. Used to keep track of alive streams for writing
    write_streams: Arc<Mutex<HashMap<String, tcp::OwnedWriteHalf>>>,
    // Holds all persistent user data including authentication data and message histories
    database: Arc<Mutex<Connection>>,
    // Holds session tokens for authenticated users. Every incoming message is referenced against this
    session_tokens: Arc<Mutex<HashMap<String, String>>>,
}

impl Server {

    /// Creates a server instance, sets up a database connection and reads config values.
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

        // Make sure that we have a message history table for every channel
        let channels: &Vec<toml::Value> = config["channels"].as_array().ok_or("Invalid channel configuration")?;
        for channel in channels {
            let channel = channel.as_str().ok_or("Invalid channel name")?;
            connection.execute(&format!("CREATE TABLE IF NOT EXISTS {channel}
                (
                    id INTEGER PRIMARY KEY,
                    username TEXT NOT NULL,
                    body TEXT NOT NULL,
                    timestamp INTEGER NOT NULL,
                    embed_pointer INTEGER,
                    embed_type TEXT
                );"),())?;
        }

        // Create a table of server users (usernames that have logged in at least once)
        connection.execute("CREATE TABLE IF NOT EXISTS user_hashes (
            username TEXT PRIMARY KEY,
            hash INTEGER UNIQUE
        );", ())?;

        let database = Arc::new(Mutex::new(connection));
        Ok(Server {
            config,
            write_streams: Arc::new(Mutex::new(HashMap::new())),
            database,
            session_tokens: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Starts the server by launching all continuously running processes
    async fn start(mut self) -> Result<(), Box<dyn Error>>{
        let listener = TcpListener::bind(self.config["listen_address"].as_str().ok_or("Invalid address")?).await?;

        let (text_tx_raw, text_rx_raw) = mpsc::channel::<TextMessage>(1);
        let (text_tx_processed, text_rx_processed) = mpsc::channel::<TextMessage>(1);
        // The command channel needs the address of the caller for sending targeted data instead of broadcasting
        let (command_tx, command_rx) = mpsc::channel::<CommandMessage>(1);

        self.begin_listen(listener, text_tx_raw, command_tx);
        self.begin_broadcast(text_rx_processed);
        tokio::spawn(process_and_save(self.database.clone(),text_rx_raw,text_tx_processed));
        self.begin_parse_commands(command_rx);
        Ok(())
    }

    /// Starts a process to listen for incoming connections. If the connection is valid and successfully authenticates,
    /// starts a process to accept messages from the stream. Otherwise the connection is dropped
    fn begin_listen(&mut self, listener: TcpListener, text_tx: mpsc::Sender<TextMessage>, command_tx: mpsc::Sender<CommandMessage>) {
        let write_streams = self.write_streams.clone();
        let db = self.database.clone();
        let tokens = self.session_tokens.clone();
        tokio::spawn(async move {
            // Endlessly accept incoming TCP connections
            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(val) => val, 
                    Err(_) => {
                        eprintln!("Failed to accept incoming TCP stream");
                        continue;
                    }
                };
                let (read_stream, mut write_stream) = stream.into_split();
                let mut reader = BufReader::new(read_stream);
                // Wait for an authentication message and authenticate the user
                let response = match authenticate(&mut reader, db.clone(), tokens.clone()).await {
                    // If the user passes, get the response
                    Ok(Some(response)) => response,
                    // If the user does not pass authentication, drop them
                    Ok(None) => {continue},
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };
                // Write the response to the user, drop them if it fails
                let username = response.username();
                let response_string = to_string(&response).unwrap() + "\n";
                if write_stream.write_all(response_string.as_bytes()).await.is_err() {
                    continue;
                }
                let mut writes_locked = write_streams.lock().await;
                writes_locked.insert(username.clone(), write_stream);
                let db_locked = db.lock().await;
                let hashed = hash_username(&username);
                let query = &format!("INSERT OR IGNORE INTO user_hashes (username, hash) VALUES('{}', {});", username, hashed);
                db_locked.execute(
                    &query,
                    ()).unwrap();
                tokio::spawn(listen_messages(reader, tokens.clone(), text_tx.clone(), command_tx.clone()));
            }
        });
    }

    /// Starts a task to broadcast messages to all server users
    fn begin_broadcast(&self, mut message_rx: mpsc::Receiver<TextMessage>) {
        let write_streams = self.write_streams.clone();
        tokio::spawn(async move {
            // While the text message channel is open, receive new messages
            while let Some(message) = message_rx.recv().await {
                print!("<{}> {}", message.username, message.body);
                let channel = message.channel.clone();
                let in_dm_channel = is_dm(&channel);
                let message = Message::Text(message);
                let mut bad_users: Vec<String> = Vec::new(); // Users that fail to communicate will be purged
                let mut streams_locked = write_streams.lock().await;
                // Iterate through all users and send the received message
                for (username, stream) in streams_locked.iter_mut() {
                    // If the user is not in the dm channel, don't broadcast to them
                    if in_dm_channel && !allowed_dm(&channel, &username) {
                        continue;
                    }
                    // If the message is valid JSON, were good. If it's cringe, don't bother sending it
                    let outgoing_message = match to_string(&message) {
                        Ok(data) => data,
                        Err(_) => continue,
                    };
                    let outgoing_message = outgoing_message + "\n";
                    // Write to the current stream
                    // If you're good nothing happens, if it fails you go on the cringe list!
                    match stream.write_all(outgoing_message.as_bytes()).await {
                        Ok(_) => {},
                        Err(_) => {
                            let _res = stream.shutdown().await;
                            bad_users.push(username.clone())
                        }
                    }
                }
                // Purge the cringe list
                streams_locked.retain(|addr, _stream| !bad_users.contains(&addr));
            }
        });
    }

    /// Start a task to parse commands and send them off to be handled
    fn begin_parse_commands(&self, mut command_rx: mpsc::Receiver<CommandMessage>) {
        let write_streams = self.write_streams.clone();
        let db = self.database.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            while let Some(cmd) = command_rx.recv().await {
                match cmd.command_type.as_str() {
                    // User requests message history
                    "history" => {
                        // Get the arguments
                        let (channel, count) = (cmd.args[0].clone(), cmd.args[1].clone());
                        let count = match count.parse() {
                            Ok(res) => res,
                            Err(_) => continue // We will not crashing the thread over a malformed command thank you very much
                        };
                        // Fetch and send the history
                        match retrieve_history(&cmd.username, db.clone(), channel, count, write_streams.clone()).await {
                            Ok(_) => {},
                            Err(error) => eprintln!("{:?}", error)
                        }
                    }
                    //User requests a list of channels
                    "channels" => {
                        match retrive_channels(address, write_streams.clone(), &config).await {
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

#[tokio::main]async fn main() {
    let config = std::fs::read_to_string("config.toml").expect("Failed to read config file");
    let server = Server::new(config).expect("Failed to start server");
    server.start().await.expect("Failed to start server");
    loop {}
}

/// Listen for incoming messages on a stream and send them off to the right process
async fn listen_messages(
    mut reader: BufReader<tcp::OwnedReadHalf>,
    tokens: Arc<Mutex<HashMap<String, String>>>,
    text_tx: mpsc::Sender<TextMessage>,
    command_tx: mpsc::Sender<CommandMessage>,
) {
    use shared::Message as m;
    let mut buffer = String::new();
    loop {
        buffer.clear();
         // If we receive some nonsense or the stream is dead, end the task
         // Dropping the stream will shut down the read half
        if reader.read_line(&mut buffer).await.is_err() {
            return;
        }
        let message = match from_str::<Message>(&buffer) {
            Ok(data) => data,
            Err(_) => return 
        };
        // Check that the message has a valid session token
        if !is_authentic(message.clone(), tokens.clone()).await {
            eprintln!("User {} provided an invalid session token, disconnecting...", message.username());
            return
        }
        // Check the type of the message and send the contents to the right process
        match message {
            m::Text(content) => {text_tx.send(content).await.unwrap();}, // Send off to save in database and broadcast
            m::Command(content) => {command_tx.send(content).await.unwrap();},
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
    // Await messages to process and send
    while let Some(mut msg) = text_rx_raw.recv().await {
        // Check if the channel is a DM channel for special treatments
        if is_dm(&msg.channel) && !channel_exists(&msg.channel, db.clone()).await? {
            // If the channel name is invalid, ignore the message
            if !is_valid_dm(&msg.channel, db.clone()).await? {
                continue;
            }
            // Else ensure the DM channel exists
            let db_locked = db.lock().await;
            db_locked.execute(&format!("CREATE TABLE IF NOT EXISTS {channel}
                (
                    id INTEGER PRIMARY KEY,
                    username TEXT NOT NULL,
                    body TEXT NOT NULL,
                    timestamp INTEGER NOT NULL,
                    embed_pointer INTEGER,
                    embed_type TEXT
                );", channel = msg.channel),())?;
        }

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
            INSERT INTO {channel} (username, body, timestamp)
            VALUES('{username}', '{body}', '{timestamp}');
            ", username = msg.username, body = msg.body, timestamp = msg.timestamp, channel = msg.channel);
            let db_locked = db.lock().await;
            match db_locked.execute(&query, ()) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("User {} attempted to access invalid channel {}, discarding...", msg.username, msg.channel);
                    continue;
                }
            }

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
    username: &str,
    db: Arc<Mutex<Connection>>,
    channel: String,
    count: usize,
    streams: Arc<Mutex<HashMap<String, tcp::OwnedWriteHalf>>>
) -> rusqlite::Result<()> {
    // If the user is not allowed in the DMs, deny accesss
    if is_dm(&channel) && !allowed_dm(&channel, &username) {
        println!("User {} tried to acces DM channel {} history without permission", username, channel);
        return Ok(());
    }
    // Get the messages from the database
    let messages = {
        let db_locked = db.lock().await;
        let mut statement = db_locked.prepare(&format!("SELECT * FROM {channel} ORDER BY id DESC LIMIT {count};"))?;
        let mut msgs = statement.query_map((), |row| {
            // The query_map *has* to return an iterator of rusql::Result
            Ok(Message::Text(TextMessage {
                message_id: row.get(0)?,
                username: row.get(1)?,
                body: row.get(2)?,
                channel: channel.clone(),
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
    let stream = streams_locked.get_mut(username).unwrap();
    for message in messages {
        let serialized = to_string(&message).unwrap() + "\n";
        let _res = stream.write_all(&serialized.as_bytes()).await;
    }
    Ok(())
}

// Retrieve a list of the channels in the server
async fn retrive_channels(
    address: std::net::SocketAddr,
    streams: Arc<Mutex<HashMap<std::net::SocketAddr, tcp::OwnedWriteHalf>>>,
    config: &Table
) -> Result<(), Box<dyn Error>> {
    let channels = config["channels"].as_array().ok_or("Invalid channel configuration")?;

    let mut streams_locked = streams.lock().await;
    if let Some(stream) = streams_locked.get_mut(&address) {
        let channels = to_string(channels)?;
        let outgoing_msg = Message::Info(InfoMessage{
           header: "channels\n".to_owned(),
           data: channels 
        });
        let serialized = to_string(&outgoing_msg)? + "\n";
        stream.write_all(serialized.as_bytes()).await?;
    }
    Ok(())
}

/// Authenticate new connections
/// If the user already exists, references their password against the database
/// If the user is new, adds them to said database
/// Returns a response message if authentication is successful
async fn authenticate(
    reader: &mut BufReader<tcp::OwnedReadHalf>,
    db: Arc<Mutex<Connection>>,
    session_tokens: Arc<Mutex<HashMap<String, String>>>
) -> Result<Option<Message>, String> {
    use shared::Message as m;

    // Receive the authentication message
    let mut buf = String::new();
    reader.read_line(&mut buf).await.map_err(|e| e.to_string())?;
    let auth_msg = from_str::<Message>(&buf).map_err(|e| e.to_string())?;
    let auth_msg = match auth_msg {
        m::Auth(inner) => inner,
        _ => return Err("Did not receive authentication message".to_owned())
    };

    // Determine if the authentication was successful
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
        // Generate a session token
        let mut tokens_locked = session_tokens.lock().await;
        let token = rand_string(100);
        tokens_locked.insert(auth_msg.username.clone(), token.clone());
        // Build the respone
        let response = Message::Auth(AuthMessage {
            username: auth_msg.username,
            auth_token: Some(token),
            password: None
        });
        return Ok(Some(response))
    }
    Ok(None)
}

/// Generate a random string of length `len`
fn rand_string(len: usize) -> String {
    thread_rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(|byte| char::from(byte))
        .collect::<String>()
}

/// Determines whether the session token of a message matches the session token generated by the server
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


// Checks whether a DM channel contains two valid users that have been on the server
async fn is_valid_dm(channel: &str, db: Arc<Mutex<Connection>>) -> rusqlite::Result<bool>{
    // Get a vec of hashes from the channel name
    let mut users = Vec::new();
    let channel = channel.replace("DM_", "");
    let users_iter = channel.split("_");
    for user in users_iter {
        match user.parse() {
            Ok(parsed) => users.push(parsed),
            Err(_) => return Ok(false)
        }
    }

    // Ensure that the channel name is sorted
    let mut sorted = users.clone();
    sorted.sort();
    if users != sorted {
        return Ok(false)
    }

    // Get all hashes from the database
    let db_locked = db.lock().await;
    let mut statement = db_locked.prepare("SELECT hash FROM user_hashes;")?;
    let response = statement.query_map((), |row| {
        row.get::<usize, i64>(0)
    })?;
    let mut hashes = Vec::new();
    for row in response {
        let row = row.unwrap();
        hashes.push(row);
    }

    // Determine whether all hashes in the channel name are users of the server
    let all_hashes_valid = users.into_iter()
        .map(|user| hashes.contains(&user))
        .fold(true, |acc, value| acc && value);

    Ok(all_hashes_valid)
}

// Returns whether or not a channel name is a DM channel
fn is_dm(channel: &str) -> bool {
    channel.starts_with("DM_")
}

// Returns whether or not a user is allowed in a DM channel
fn allowed_dm(channel: &str, username: &str) -> bool {
    let hashed = hash_username(username);
    channel.contains(&hashed.to_string())
}

fn hash_username(username: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    username.hash(&mut hasher);
    (hasher.finish() >> 1) as i64
}

async fn channel_exists(channel: &str, db: Arc<Mutex<Connection>>) -> rusqlite::Result<bool> {
    let db_locked = db.lock().await;
    match db_locked.query_row_and_then(
        &format!("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='{}';", channel),
        (),
        |row| row.get::<usize, i64>(0)
    ).optional()? {
        Some(_) => Ok(true),
        None => Ok(false)
    }
}