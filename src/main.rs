mod network;
mod blockchain;

#[tokio::main]
async fn main() {
    network::run_server().await;
}
