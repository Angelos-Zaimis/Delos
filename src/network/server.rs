use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};


pub async fn run_server() {
    let listener = start_listener().await;

    loop {
        let (mut socket, _) = accepting_connections(&listener).await;

        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];

            loop {
                let reader = reading_from_socket(&mut socket, &mut buffer).await;

                if reader == 0 {
                    break;
                }

                if let Err(e) = socket.write_all(&buffer[0..reader]).await {
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

async fn reading_from_socket(socket: &mut TcpStream, buffer: &mut Vec<u8>) -> usize {
    match socket.read(buffer).await {
        Ok(0) => {
            println!("Client disconnected.");
            0
        }
        Ok(n) => {
            let message = String::from_utf8_lossy(&buffer[..n]);
            println!("Received message: {}", message);
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