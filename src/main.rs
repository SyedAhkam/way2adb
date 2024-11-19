use std::os::fd::IntoRawFd;

use tokio::join;

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

    let server_task = tokio::task::spawn(async {
        server::start_server().await.unwrap();
    });

    let streamer_task = tokio::task::spawn(async move {
        pipewire::start_streaming(stream.pipe_wire_node_id(), fd)
            .await
            .unwrap();
    });

    // FIXME: use try_join!
    let (_, _) = tokio::join!(server_task, streamer_task);
}
