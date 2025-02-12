use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn run_server() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

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
