use crate::voice::VoiceHandler;
use async_std::net::TcpStream;
use async_std::prelude::*;
use futures::{select, FutureExt};
use std::io::{self, Write};
use std::path::PathBuf;

struct Client {
    stream: TcpStream,
    voice_handler: Option<VoiceHandler>,
    username: String,
    token: String,
}

impl Client {
    async fn new(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            stream,
            voice_handler: None,
            username: String::new(),
            token: String::new(),
        })
    }

    async fn authenticate(&mut self) -> Result<()> {
        print!("Username: ");
        io::stdout().flush()?;
        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        
        print!("Password: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;

        let auth_msg = Message::Auth {
            username: username.trim().to_string(),
            password: password.trim().to_string(),
        };

        self.stream
            .write_all(serde_json::to_string(&auth_msg)?.as_bytes())
            .await?;

        // Receive token
        let mut buffer = [0u8; 1024];
        let n = self.stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]);
        self.token = response.trim().to_string();
        self.username = username.trim().to_string();

        Ok(())
    }

    async fn send_file(&mut self, path: &str) -> Result<()> {
        let path = PathBuf::from(path);
        let filename = path.file_name()
            .ok_or("Invalid filename")?
            .to_string_lossy()
            .into_owned();

        let data = fs::read(&path).await?;
        let chunk_size = 1024 * 1024; // 1MB chunks
        let total_chunks = (data.len() as f64 / chunk_size as f64).ceil() as u32;

        for (i, chunk) in data.chunks(chunk_size).enumerate() {
            let msg = Message::FileTransfer {
                from: self.username.clone(),
                filename: filename.clone(),
                data: chunk.to_vec(),
                chunk_id: i as u32,
                total_chunks,
            };

            self.stream
                .write_all(serde_json::to_string(&msg)?.as_bytes())
                .await?;
        }

        Ok(())
    }

    async fn join_voice(&mut self) -> Result<()> {
        let voice_handler = VoiceHandler::new()?;
        let (tx, mut rx) = mpsc::unbounded();

        let mut voice_handler = voice_handler;
        voice_handler.start_recording(tx)?;
        self.voice_handler = Some(voice_handler);

        let username = self.username.clone();
        let mut stream = self.stream.clone();

        // Send voice join message
        let join_msg = Message::VoiceJoin {
            username: username.clone(),
        };
        stream
            .write_all(serde_json::to_string(&join_msg)?.as_bytes())
            .await?;

        // Handle voice data
        tokio::spawn(async move {
            let mut sequence = 0u32;
            while let Some(data) = rx.next().await {
                let msg = Message::VoiceData {
                    from: username.clone(),
                    data,
                    sequence,
                };
                if let Err(e) = stream
                    .write_all(serde_json::to_string(&msg)?.as_bytes())
                    .await
                {
                    eprintln!("Error sending voice data: {}", e);
                    break;
                }
                sequence = sequence.wrapping_add(1);
            }
        });

        Ok(())
    }

    async fn send_direct_message(&mut self, to: &str, content: &str) -> Result<()> {
        let dm = Message::DirectMessage {
            from: self.username.clone(),
            to: to.to_string(),
            content: content.to_string(),
        };

        self.stream
            .write_all(serde_json::to_string(&dm)?.as_bytes())
            .await?;

        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        let mut stdin = async_std::io::stdin();
        let mut stdout = io::stdout();
        let mut buffer = [0u8; 1024];
        let mut input = String::new();

        println!("Commands:");
        println!("/dm <username> <message> - Send direct message");
        println!("/file <path> - Send file");
        println!("/voice - Join voice chat");
        println!("/quit - Exit");

        let stream_clone = self.stream.clone();
        let mut stream_clone = stream_clone;

        loop {
            select! {
                result = stream_clone.read(&mut buffer).fuse() => {
                    match result {
                        Ok(n) if n == 0 => break,
                        Ok(n) => {
                            let message = String::from_utf8_lossy(&buffer[..n]);
                            if let Ok(msg) = serde_json::from_str::<Message>(&message) {
                                match msg {
                                    Message::VoiceData { from, data, sequence: _ } => {
                                        if let Some(voice_handler) = &mut self.voice_handler {
                                            if let Ok(pcm_data) = voice_handler.decode_voice_data(&data) {
                                                // Play the decoded audio using cpal
                                                // Implementation would go here
                                            }
                                        }
                                    },
                                    _ => println!("{}", message),
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from server: {}", e);
                            break;
                        }
                    }
                }
                result = stdin.read_line(&mut input).fuse() => {
                    match result {
                        Ok(_) => {
                            let input = input.trim();
                            if input.is_empty() {
                                continue;
                            }

                            if input.starts_with("/") {
                                let parts: Vec<&str> = input.splitn(3, ' ').collect();
                                match parts[0] {
                                    "/dm" => {
                                        if parts.len() == 3 {
                                            self.send_direct_message(parts[1], parts[2]).await?;
                                        } else {
                                            println!("Usage: /dm <username> <message>");
                                        }
                                    }
                                    "/file" => {
                                        if parts.len() == 2 {
                                            self.send_file(parts[1]).await?;
                                        } else {
                                            println!("Usage: /file <path>");
                                        }
                                    }
                                    "/voice" => {
                                        self.join_voice().await?;
                                        println!("Joined voice chat!");
                                    }
                                    "/quit" => break,
                                    _ => println!("Unknown command"),
                                }
                            } else {
                                let msg = Message::Chat {
                                    username: self.username.clone(),
                                    content: input.to_string(),
                                };
                                self.stream
                                    .write_all(serde_json::to_string(&msg)?.as_bytes())
                                    .await?;
                            }
                            input.clear();
                        }
                        Err(e) => {
                            eprintln!("Error reading from stdin: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("127.0.0.1:8080").await?;
    
    println!("Welcome to Clear Comm!");
    client.authenticate().await?;
    println!("Authentication successful.");
    
    client.run().await?;
    
    Ok(())
}
