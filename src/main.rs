use std::os::fd::IntoRawFd;

use tokio::{join, sync::mpsc};

mod message;
mod pipewire;
mod portal;
mod server;

#[tokio::main]
async fn main() {
    let (stream, fd) = portal::open_portal().await.expect("failed to open portal");
    println!(
        "node id {}, fd {}",
        stream.pipe_wire_node_id(),
        fd.try_clone().unwrap().into_raw_fd()
    );

    let (tx, rx) = mpsc::channel::<message::StreamMessage>(16);

    let server_task = tokio::task::spawn(async {
        server::start_server(rx).await.unwrap();
    });

    let streamer_task = tokio::task::spawn(async move {
        pipewire::start_streaming(stream.pipe_wire_node_id(), fd, tx)
            .await
            .unwrap();
    });

    if let Err(e) = tokio::try_join!(server_task, streamer_task) {
        eprintln!("an error occurred: {}", e);
    }
}
