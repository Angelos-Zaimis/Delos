use rocksdb::{Options, DB};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::blockchain::Block;




#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub address: String,
}

pub struct  PeerManager {
    peers: Vec<Peer>,
    db: Arc<DB>
}


pub type SharedPeerManager = Arc<Mutex<PeerManager>>;
impl PeerManager {
    pub fn new(db: Arc<DB>) -> Self {
        PeerManager {
            peers: Vec::new(),
            db,
        }
    }

    pub fn add_peer(&self, peer: &Peer) {
        let mut all_peers = self.get_all_peers();

        if all_peers.iter().any(|p| p.id == peer.id) {
            println!("⚠Peer {} is already known, skipping storage.", peer.address);
            return;
        }

        let new_peer = serde_json::to_string(peer).expect("Failed to serialize new peer");
        self.db.put(&peer.id, new_peer).unwrap();

        println!("Stored new peer: {}", peer.address);

    }

    pub fn get_peer(&self, peer_id: &str) -> Option<Peer> {
        if let Ok(Some(peer)) = self.db.get(peer_id) {
            serde_json::from_slice(&peer).ok()
        } else {
            None
        }
    }

    pub fn get_all_peers(&self) -> Vec<Peer> {
        let mut peers = Vec::new();
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);

        for item in iter {
            if let Ok((_, value)) = item {
                if let Ok(peer) = serde_json::from_slice::<Peer>(&value) {
                    peers.push(peer);
                }
            }
        }
        peers
    }

    pub fn store_peers(&self, peers: &Vec<Peer>) {
        for peer in peers.iter() {
            self.add_peer(peer);
        }
    }

    pub fn remove_peer(&self, peer_id: &str) {
        self.db.delete(peer_id).unwrap_or_else(|e| {
            eprintln!("Couldnt find peer to remove: {}", e);
        })
    }

    pub async fn connect_to_peer(&self, address: &str) {
        let mut socket = self.connect_to_server(address).await;


        let all_peers = self.get_all_peers();

        if let Err(e) = self.send_peers(&mut socket, &all_peers).await {
            eprintln!("Failed to send peer list: {}", e);
            return;
        }

        let mut buffer = vec![0; 1024];

        let reader = match self.start_reading(&mut socket, &mut buffer).await {
            Ok(size) => size,
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                return;
            }
        };

        let received_data = String::from_utf8_lossy(&buffer[..reader]);

        println!("Received from server: {}", received_data);

        if let Ok(received_peers) = serde_json::from_str::<Vec<Peer>>(&received_data) {
            for peer in received_peers {
                if peer.address != address {
                    println!("Discovered new peer: {}", peer.address);
                    self.add_peer(&peer);
                }
            }
        } else {
            println!("Received invalid peer data");
        }
    }

    pub async fn send_peers(&self, socket: &mut TcpStream, peers: &Vec<Peer>) -> Result<(), std::io::Error> {
        let serialized_peers = serde_json::to_string(peers).expect("Failed to serialize peers");
        socket.write_all(serialized_peers.as_bytes()).await?;
        println!("Sent peer list: {}", serialized_peers);
        Ok(())
    }

    pub async fn start_reading(&self, socket: &mut TcpStream, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        socket.read(buffer).await
    }

    pub async fn connect_to_server(&self, address: &str) -> tokio::net::TcpStream {
        TcpStream::connect(address).await.expect("Failed to connect to server")
    }

    pub async fn connect_to_server_static(address: &str) -> tokio::net::TcpStream {
        TcpStream::connect(address).await.expect("Failed to connect to server")
    }

    pub async fn start_reading_static(socket: &mut TcpStream, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        socket.read(buffer).await
    }
}