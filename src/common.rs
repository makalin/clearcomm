use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Auth {
        username: String,
        password: String,
    },
    Join {
        username: String,
        token: String,
    },
    Chat {
        username: String,
        content: String,
    },
    DirectMessage {
        from: String,
        to: String,
        content: String,
    },
    FileTransfer {
        from: String,
        filename: String,
        data: Vec<u8>,
        chunk_id: u32,
        total_chunks: u32,
    },
    VoiceData {
        from: String,
        data: Vec<u8>,
        sequence: u32,
    },
    VoiceJoin {
        username: String,
    },
    VoiceLeave {
        username: String,
    },
    Leave {
        username: String,
    },
}