use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

async fn process_socket(mut socket: TcpStream) -> std::io::Result<()> {
    println!("new connection");

    socket.write_all(b"hello").await?;

    Ok(())
}

// FIXME: migrate to a udp implementation later
pub async fn start_server() -> std::io::Result<()> {
    println!("starting tcp server");
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async {
            process_socket(socket).await;
        });
    }
}
