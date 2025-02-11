use sha2::{Sha256, Digest};
use std::fmt::Write;

#[derive(Debug)]
pub struct Block {
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String
}

impl Block {
    pub fn new(timestamp: i64, data: String, previous_hash: String) -> Self {
        let mut block = Block {
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
        };
        block.hash = Block::calculate_hash(&block);
        block
    }

    fn calculate_hash(block: &Block) -> String {
        let mut hash = Sha256::new();
        hash.update(block.timestamp.to_string().as_bytes());
        hash.update(block.data.as_bytes());
        hash.update(block.previous_hash.as_bytes());
        let final_hash = hash.finalize();

        let mut hash_str = String::new();
        for byte in final_hash {
            write!(&mut hash_str, "{:02x}", byte).expect("Unable to write");
        }
        hash_str
    }
}