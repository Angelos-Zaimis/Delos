use crate::network::{run_server, PeerManager};

mod network;
mod blockchain;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "peers" {
        PeerManager::connect_to_peer("127.0.0.1:6000", "Hello there").await;
    } else {
        run_server().await;
    }
}
