use crate::auth::Auth;
use crate::file_handler::FileHandler;
use crate::voice::VoiceHandler;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use futures::channel::mpsc;
use futures::{select, FutureExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct Server {
    clients: Arc<Mutex<HashMap<String, Sender>>>,
    auth: Arc<Auth>,
    file_handler: Arc<FileHandler>,
    voice_rooms: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl Server {
    fn new(auth: Auth, file_handler: FileHandler) -> Self {
        Server {
            clients: Arc::new(Mutex::new(HashMap::new())),
            auth: Arc::new(auth),
            file_handler: Arc::new(file_handler),
            voice_rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_message(&self, msg: Message, username: &str) -> Result<()> {
        match msg {
            Message::DirectMessage { from, to, content } => {
                if let Some(client) = self.clients.lock().await.get(&to) {
                    let dm = Message::DirectMessage {
                        from: from.clone(),
                        to: to.clone(),
                        content,
                    };
                    client.unbounded_send(serde_json::to_string(&dm)?)?;
                }
            }
            Message::FileTransfer { from, filename, data, chunk_id, total_chunks } => {
                let is_complete = self.file_handler
                    .save_file_chunk(&filename, &data, chunk_id, total_chunks)
                    .await?;

                if is_complete {
                    self.broadcast(
                        &from,
                        &format!("{} shared file: {}", from, filename),
                    ).await;
                }
            }
            Message::VoiceJoin { username } => {
                let mut voice_rooms = self.voice_rooms.lock().await;
                voice_rooms
                    .entry("general".to_string())
                    .or_default()
                    .push(username.clone());
            }
            Message::VoiceData { from, data, sequence } => {
                let voice_rooms = self.voice_rooms.lock().await;
                if let Some(users) = voice_rooms.get("general") {
                    for user in users {
                        if user != &from {
                            if let Some(client) = self.clients.lock().await.get(user) {
                                let voice_msg = Message::VoiceData {
                                    from: from.clone(),
                                    data: data.clone(),
                                    sequence,
                                };
                                client.unbounded_send(serde_json::to_string(&voice_msg)?)?;
                            }
                        }
                    }
                }
            }
            _ => {
                self.broadcast(username, &serde_json::to_string(&msg)?).await;
            }
        }
        Ok(())
    }

}
