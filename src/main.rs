use std::os::fd::IntoRawFd;

use tokio::{join, sync::mpsc};

mod adb;
mod encoder;
mod message;
mod pipewire;
mod portal;
mod server;

const TCP_PORT: u16 = 8080; // FIXME: pick a good one

#[tokio::main]
async fn main() {
    // ADB reverse
    adb::reverse_port_adb().expect("failed to reverse port with adb");

    // Open xdg portal
    let (stream, fd) = portal::open_portal().await.expect("failed to open portal");
    println!(
        "node id {}, fd {}",
        stream.pipe_wire_node_id(),
        fd.try_clone().unwrap().into_raw_fd()
    );

    // Create channel for comm b/w streamer and tcp server
    let (tx, rx) = mpsc::channel::<message::StreamMessage>(16);

    // Spawn the tcp server
    let server_task = tokio::task::spawn(async {
        server::start_server(rx).await.unwrap();
    });

    // Spawn the streamer
    let streamer_task = tokio::task::spawn(async move {
        pipewire::start_streaming(stream.pipe_wire_node_id(), fd, tx)
            .await
            .unwrap();
    });

    // Wait for them to finish / err
    if let Err(e) = tokio::try_join!(server_task, streamer_task) {
        eprintln!("an error occurred: {}", e);
    }
}
