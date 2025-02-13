use bincode;
use crate::blockchain::{Block, Blockchain};
use crate::network::peers::PeerManager;

pub struct Synchronizer {
    blockchain: Blockchain,
    peer_manager: PeerManager
}

impl Synchronizer {
    pub fn new(blockchain: Blockchain, peer_manager: PeerManager) -> Synchronizer {
        Synchronizer {
            blockchain,
            peer_manager
        }
    }

    pub async fn synchronize(&mut self) {

        fn deserialize_block(data: &[u8]) -> Option<Block> {
            match bincode::deserialize::<Block>(data) {
                Ok(block) => Some(block),
                Err(_) => None,
            }
        }
    }
}