use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn run_server() {

    let listener = match TcpListener::bind("127.0.0.17879").await {
        Ok(listener ) => listener,
        Err(e) => {
            eprintln!("Failed to bind to port: {}", e);
            std::process::exit(1);
        }    };

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];

            loop {
                let reader = socket.read(&mut buffer).await.unwrap();
                if reader == 0 {
                    break;
                }
                socket.write_all(&buffer[0..reader]).await.unwrap();
            }
        });
    }
}
