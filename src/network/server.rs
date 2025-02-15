use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::network::PeerManager;
use crate::network::peers::Peer;

pub async fn run_server( peer_manager: &mut PeerManager) {
    let listener = start_listener().await;

    loop {
        let (mut socket, _) = accepting_connections(&listener).await;

        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];

            loop {
                let reader = reading_from_socket(&mut socket, &mut buffer,peer_manager).await;

                if reader == 0 {
                    break;
                }

                let known_peers = peer_manager.get_all_peers();
                let serialized_peers = serde_json::to_string(&known_peers).expect("Failed to serialize peers");

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
                let peers = &received_peers;
                for peer in received_peers {
                    peer_manager.add_peer(&peer);
                }

                println!("Added {} new peers", &peers.len());
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