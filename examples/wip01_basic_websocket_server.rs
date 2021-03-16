use futures_util::{StreamExt};
use log::*;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite;

/// We don't use `#[tokio::main]` because we want to easily be able to transfer this executable-based example into a library-based one (that won't *have* a main function).
fn main() {
  let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
  tokio_runtime.block_on(async_main())
}

/// The "actual" main function, which is async and run by tokio.
async fn async_main() {
  env_logger::init();

  let addr = "127.0.0.1:10202";
  let listener = TcpListener::bind(&addr).await.expect("Failed to bind to address");
  info!("Listening on: {}", addr);

  while let Ok((stream, _)) = listener.accept().await {
    let peer = stream.peer_addr().expect("Connected streams should have a peer address");
    info!("Peer address: {}", peer);

    tokio::spawn(accept_connection(peer, stream));
  }
}

async fn accept_connection(_peer: SocketAddr, stream: TcpStream) {
  let addr = stream.peer_addr().expect("Connected streams should have a peer address");
  info!("Peer address: {}", addr);

  let ws_stream = tokio_tungstenite::accept_async(stream)
      .await
      .expect("Error during the websocket handshake occurred");

  info!("New WebSocket connection: {}", addr);

  let (write, read) = ws_stream.split();
  read.forward(write).await.expect("Failed to forward message");
}
