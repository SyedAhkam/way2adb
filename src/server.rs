use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc},
};

use crate::message::{StreamMessage, TcpMessage};

async fn process_socket(
    mut socket: TcpStream,
    mut rx: broadcast::Receiver<TcpMessage>,
) -> std::io::Result<()> {
    println!("New connection");

    socket.write_all(b"welcome :)\n").await?;

    let (mut socket_r, mut socket_w) = socket.into_split();
    let reader_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let TcpMessage::Frame(frame_bytes) = msg; // expecting only one type of message

            socket_w.write_all(&frame_bytes).await;
        }
    });

    let mut buf = [0; 1024];
    loop {
        let n = socket_r.read(&mut buf).await?;
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
pub async fn start_server(mut rx_stream: mpsc::Receiver<StreamMessage>) -> std::io::Result<()> {
    println!("Starting TCP server..");

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, _) = broadcast::channel::<TcpMessage>(32);

    let tx_cloned = tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx_stream.recv().await {
            println!("Message from streamer: {:?}", msg);
            if tx_cloned.receiver_count() > 0 {
                match msg {
                    StreamMessage::Frame(v) => tx_cloned.send(TcpMessage::Frame(v)),
                    StreamMessage::Connected => Ok(0), // TODO
                }
                .unwrap();
            }
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
