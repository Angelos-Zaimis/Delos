use sha2::{Sha256, Digest};
use std::fmt::Write;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::digest::Update;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub previous_hash: String,
    pub hash: String,
    pub data: String,
    pub nonce: u64,
    pub difficulty: u32,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, data: String, difficulty: u32) -> Self {
        let timestamp  = Self::current_timestamp();
        let nonce: u64 = 0;
        let hash = Block::calculate_hash(index, &timestamp, &previous_hash, &data, nonce);

        Self {
            index,
            timestamp,
            previous_hash,
            hash,
            data,
            nonce,
            difficulty
        }
    }

    pub fn calculate_hash(index: u64, &timestamp: &str, previous_hash: &str, &data: &str, nonce: u64) -> String {
        let mut hash = Sha256::new();
        hash.update(index.to_le_bytes());
        hash.update(timestamp.to_string().as_bytes());
        hash.update(previous_hash.as_bytes());
        hash.update(data.as_bytes());
        hash.update(nonce.to_le_bytes());
        let final_hash = hash.finalize();

        let mut hash_str = String::new();
        for byte in final_hash {
            write!(&mut hash_str, "{:02x}", byte).expect("Unable to write");
        }
        hash_str
    }

    fn current_timestamp() -> String {
        Utc::now().timestamp().to_string()
    }
}