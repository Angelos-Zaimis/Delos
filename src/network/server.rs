use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::network::PeerManager;
use crate::network::peers::{Peer, SharedPeerManager};
use std::sync::Arc;

pub async fn run_server(peer_manager: SharedPeerManager) {
    let listener = start_listener().await;
    println!("Listening at port 6000...");

    loop {
        let (mut socket, _) = accepting_connections(&listener).await;
        let peer_manager_clone = Arc::clone(&peer_manager);

        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];

            loop {
                let reader = {
                    let mut peer_manager_lock = peer_manager_clone.lock().await;
                    reading_from_socket(&mut socket, &mut buffer, &mut peer_manager_lock).await
                };

                if reader == 0 {
                    break;
                }

                let known_peers = {
                    let peer_manager_lock = peer_manager_clone.lock().await;
                    peer_manager_lock.get_all_peers()
                };

                let serialized_peers = serde_json::to_string(&known_peers).expect("Failed to serialize peers");

                println!("Sending peer list to client: {:?}", &serialized_peers);

                if let Err(e) = socket.write_all(serialized_peers.as_bytes()).await {
                    eprintln!("Write failed: {}", e);
                    break;
                }
            }
        });
    }
}


async fn start_listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:6000")
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to port: {}", e);
            std::process::exit(1);
        })
}

async fn reading_from_socket(socket: &mut TcpStream, buffer: &mut Vec<u8>, peer_manager: &mut PeerManager) -> usize {
    match socket.read(buffer).await {
        Ok(0) => {
            println!("Client disconnected.");
            0
        }
        Ok(n) => {
            let received_data = String::from_utf8_lossy(&buffer[..n]);
            println!("Received message: {}", received_data);

            if let Ok(received_peers) = serde_json::from_str::<Vec<Peer>>(&received_data) {

                for peer in received_peers {
                    println!("Adding new peer: {}", peer.address);
                    peer_manager.add_peer(&peer);
                }

                println!("Added new peers");
            } else {
                println!("⚠️ Received invalid peer data");
            }
            n
        }
        Err(e) => {
            eprintln!("Failed to read from socket: {}", e);
            0
        }
    }
}


async fn accepting_connections(listener: &TcpListener) -> (TcpStream, std::net::SocketAddr) {
    listener.accept().await.expect("Failed to accept connection")
}