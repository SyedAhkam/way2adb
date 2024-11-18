use std::os::fd::IntoRawFd;

mod pipewire;
mod portal;

#[tokio::main]
async fn main() {
    let (stream, fd) = portal::open_portal().await.expect("failed to open portal");
    println!(
        "node id {}, fd {}",
        stream.pipe_wire_node_id(),
        fd.try_clone().unwrap().into_raw_fd()
    );

    if let Err(e) = pipewire::start_streaming(stream.pipe_wire_node_id(), fd).await {
        eprintln!("Error: {}", e);
    };
}
