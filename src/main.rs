#![allow(warnings)]

use std::sync::Arc;
use rocksdb::{Options, DB};
use tokio::sync::Mutex;
use crate::network::{run_server, PeerManager, };
use crate::network::peers::{Peer, SharedPeerManager};

mod network;
mod blockchain;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let db_path = if args.len() > 2 {
        format!("data/peers_db_{}", args[2])
    } else {
        "data/peers_db".to_string()
    };

    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = Arc::new(DB::open(&opts, db_path).expect("Failed to open RocksDB for peers"));

    let peer_manager: Arc<Mutex<PeerManager>> = Arc::new(Mutex::new(PeerManager::new(db.clone())));

    if args.len() > 1 && args[1] == "peers" {
        let peer_id = format!("peer_{}", args[2]);
        let peer_address = format!("127.0.0.1:{}", 6000 + args[2].parse::<u16>().unwrap_or(1));

        let peer_manager_clone = Arc::clone(&peer_manager);

        tokio::spawn(async move {
            let mut peer_manager_lock = peer_manager_clone.lock().await;
            peer_manager_lock.add_peer(&Peer {
                id: peer_id.clone(),
                address: peer_address.clone(),
            });

            drop(peer_manager_lock);

            let peer_manager_lock = peer_manager_clone.lock().await;
            peer_manager_lock.connect_to_peer("127.0.0.1:6000").await;
        }).await.unwrap();
    } else {
        run_server(peer_manager).await;
    }
}
