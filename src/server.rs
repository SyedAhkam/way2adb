use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

async fn process_socket(
    mut socket: TcpStream,
    mut rx: broadcast::Receiver<String>,
) -> std::io::Result<()> {
    println!("New connection");

    socket.write_all(b"welcome :)\n").await?;

    let reader_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            println!("got msg");
        }
    });

    let mut buf = [0; 1024];
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            break;
        }
    }

    // Cleanup after disconnection
    reader_task.abort();
    println!("TCP connection dropped");

    Ok(())
}

// FIXME: migrate to a udp implementation later
pub async fn start_server() -> std::io::Result<()> {
    println!("Starting TCP server..");

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, _) = broadcast::channel(32);

    let tx_cloned = tx.clone();
    tokio::spawn(async move {
        loop {
            if tx_cloned.receiver_count() > 0 {
                tx_cloned.send("Hello".into()).unwrap();
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Add a delay
        }
    });

    loop {
        let (socket, _) = listener.accept().await?;
        let rx = tx.subscribe();

        tokio::spawn(async {
            if let Err(e) = process_socket(socket, rx).await {
                eprintln!("Error processing socket: {:?}", e);
            }
        });
    }
}
