use std::sync::{Arc, Mutex, MutexGuard};

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

    pub fn remove_peer(&self, address: &String) {
        let mut peers: MutexGuard<Vec<Peer>> = self.peers.lock().unwrap();

        if let Some(index) = peers.iter().position(|p| p.address == *address) {
            peers.remove(index);
        }
    }
}