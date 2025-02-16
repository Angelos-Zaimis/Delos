use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::network::PeerManager;
use crate::blockchain::Ledger;
use crate::network::peers::{Peer, SharedPeerManager};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use crate::r#enum::network_message::NetworkMessage;

pub async fn run_server(peer_manager: SharedPeerManager, ledger: Arc<Mutex<Ledger>>) {
    let listener = start_listener().await;
    println!("Listening at port 6000...");

    loop {
        let (mut socket, _) = accepting_connections(&listener).await;
        let peer_manager_clone = Arc::clone(&peer_manager);
        let ledger_clone = Arc::clone(&ledger);

        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];

            loop {
                let reader = {
                    let mut peer_manager_lock = peer_manager_clone.lock().await;
                    let mut ledger_lock = ledger_clone.lock().await;
                    reading_from_socket(&mut socket, &mut buffer, &mut peer_manager_lock, &mut ledger_lock).await
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

async fn reading_from_socket<'a>(
    socket: &mut TcpStream,
    buffer: &mut Vec<u8>,
    peer_manager: &mut MutexGuard<'a, PeerManager>,
    ledger: &mut MutexGuard<'a, Ledger>,
) -> usize {
    match socket.read(buffer).await {
        Ok(0) => {
            println!("Client disconnected.");
            return 0;
        }
        Ok(n) => {
            let received_data = String::from_utf8_lossy(&buffer[..n]);

            match serde_json::from_str::<NetworkMessage>(&received_data) {
                Ok(NetworkMessage::RequestBlockchain) => {
                    let blockchain = &ledger.chain;
                    let serialized_chain = serde_json::to_string(&NetworkMessage::SendBlockchain(blockchain.clone()))
                        .expect("Failed to serialize blockchain");

                    if let Err(e) = socket.write_all(serialized_chain.as_bytes()).await {
                        eprintln!("Failed to send blockchain data: {}", e);
                    } else {
                        println!("Sent blockchain to peer ({} blocks).", blockchain.len());
                    }
                }

                Ok(NetworkMessage::SendBlockchain(received_chain)) => {
                    if received_chain.len() > ledger.chain.len() {
                        println!("🔄 Updating blockchain from peer ({} blocks received)...", received_chain.len());
                        ledger.chain = received_chain;
                        ledger.save_chain();
                    } else {
                        println!("✔ Local blockchain is already up-to-date.");
                    }
                }

                Ok(NetworkMessage::NewTransaction(transaction, sender_public_key)) => {
                    if transaction.is_valid(&sender_public_key) {
                        ledger.add_transaction(transaction, &sender_public_key);
                        ledger.save_mempool();
                        println!("Transaction added.");
                    } else {
                        println!("Invalid transaction received.");
                    }
                }

                Ok(NetworkMessage::RequestPeers) => {
                    let all_peers = peer_manager.get_all_peers();
                    let serialized_peers = serde_json::to_string(&NetworkMessage::SendPeers(all_peers))
                        .expect("Failed to serialize peer list");

                    if let Err(e) = socket.write_all(serialized_peers.as_bytes()).await {
                        eprintln!("Failed to send peer list: {}", e);
                    }
                }

                Ok(NetworkMessage::SendPeers(received_peers)) => {
                    for peer in received_peers {
                        peer_manager.add_peer(&peer);
                    }
                    println!("🔗 Updated peer list.");
                }

                Err(_) => {
                    println!("Received unknown message type.");
                }

                _ => {}
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