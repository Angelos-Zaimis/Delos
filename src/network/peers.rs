use std::sync::{Arc, Mutex, MutexGuard};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone, Debug)]
pub struct Peer {
    pub address: String,
}

pub struct PeerManager {
    peers: Arc<Mutex<Vec<Peer>>>
}

impl PeerManager {
    pub fn new() -> Self {
        PeerManager {
            peers: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn add_peer(&self, address: String) {
        let mut peers: MutexGuard<Vec<Peer>> = self.peers.lock().unwrap();
        if !peers.iter().any(|p| p.address == address) {
            let new_peer = Peer {
                address
            };
            peers.push(new_peer);
        }
    }
    pub fn get_peers(&self) -> Vec<Peer> {
        self.peers.lock().unwrap().clone()
    }

    pub fn remove_peer(&self, address: &str) {
        let mut peers: MutexGuard<Vec<Peer>> = self.peers.lock().unwrap();

        if let Some(index) = peers.iter().position(|p| p.address == *address) {
            peers.remove(index);
        }
    }

    pub async fn connect_to_peer(address: &str, message: &str) {
        let mut socket = Self::connect_to_server(address).await;

        Self::send_message(&mut socket, message).await;

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

    async fn send_message(socket: &mut TcpStream, message: &str) {
        socket.write_all(message.as_bytes()).await.unwrap_or_else(|e| {
            eprintln!("Write failed: {}", e);
        });
        println!("Message sent: {}", message);
    }

    async fn connect_to_server(address: &str) -> TcpStream {
        TcpStream::connect(address).await.expect("Failed to connect to server")
    }
}