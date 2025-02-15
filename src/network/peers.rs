use rocksdb::{Options, DB};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub address: String,
}

pub struct PeerManager {
    peers: Vec<Peer>,
    db: DB
}

impl PeerManager {
    pub fn new() -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, "data/peers_db").expect("Failed to open RocksDB for peers");

        PeerManager {
            peers: Vec::new(),
            db
        }
    }

    pub fn add_peer(&self, peer: &Peer) {
        let new_peer = serde_json::to_string(peer).expect("Fail to serialize new peer");
        self.db.put(&peer.id, new_peer).unwrap()
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

    pub fn store_peers(&self, peers: &mut Vec<Peer>) {
        for peer in peers.iter_mut() {
            self.add_peer(peer);
        }
    }

    pub fn remove_peer(&self, peer_id: &str) {
        self.db.delete(peer_id).unwrap_or_else(|e| {
            eprintln!("Couldnt find peer to remove: {}", e);
        })
    }

    pub async fn connect_to_peer(address: &str, peer_manager: &PeerManager) {
        let mut socket = Self::connect_to_server(address).await;

        let all_peers = peer_manager.get_all_peers();

        Self::send_peers(&mut socket, &all_peers).await;

        let mut buffer = vec![0; 1024];
        let reader = Self::start_reading(&mut socket, &mut buffer).await;

        let received_message = String::from_utf8_lossy(&buffer[..reader]);
        println!("Received: {}", received_message);
    }

    async fn start_reading(socket: &mut TcpStream, buffer: &mut Vec<u8>) -> usize {
        match socket.read(buffer).await {
            Ok(0) => {
                println!("Connection closed by server");
                0
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from server: {}", e);
                0
            }
        }
    }

    async fn send_peers(socket: &mut TcpStream, peers: &Vec<Peer>) {
        let serialized_peers = serde_json::to_string(peers).expect("Failed to serialize peer");

        socket.write_all(serialized_peers.as_bytes()).await.unwrap_or_else(|e| {
            eprintln!("Write failed: {}", e);
        });

        println!("Sent peer list: {}", serialized_peers);
    }

    async fn connect_to_server(address: &str) -> TcpStream {
        TcpStream::connect(address).await.expect("Failed to connect to server")
    }
}