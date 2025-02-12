use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bincode;
use serde::{Deserialize};
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
        for peer in self.peer_manager.get_peers() {
            match TcpStream::connect(&peer.address).await {
                Ok(mut stream) => {
                    println!("Connected to peer: {}", peer.address);

                    if let Err(e) = stream.write_all(b"get_latest_block").await {
                        println!("Failed to send request to {}: {}", peer.address, e);
                        continue;
                    }

                    let mut buffer = vec![0; 1024];
                    match stream.read(&mut buffer).await {
                        Ok(n) if n > 0 => {
                            if let Some(latest_block) = self.deserialize_block(&buffer[..n]) {
                                if self.synchronize(latest_block) {
                                    println!("Block added to the blockchain!");
                                } else {
                                    println!("Received invalid block from {}", peer.address);
                                }
                            } else {
                                println!("Failed to deserialize block from {}", peer.address);
                            }
                        }
                        Ok(_) => println!("No data received from {}", peer.address),
                        Err(e) => println!("Failed to read from {}: {}", peer.address, e),
                    }
                }
                Err(e) => println!("Failed to connect to {}: {}", peer.address, e),
            }
        }

        fn deserialize_block(data: &[u8]) -> Option<Block> {
            match bincode::deserialize::<Block>(data) {
                Ok(block) => Some(block),
                Err(_) => None,
            }
        }
    }
}