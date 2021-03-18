use std::{net::SocketAddr};
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use tokio::{net::{TcpListener, TcpStream}, sync::{broadcast, mpsc, watch}};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

/// Main thread loop for running the websocket server.
///
/// This function launches a tokio runtime to handle most server functions. The function will return after the tokio runtime exits.
pub fn main(
  ser_thread_alive_tx: watch::Sender::<bool>,
  ser_msg_tx: broadcast::Sender::<Vec<tungstenite::Message>>,
  cli_msg_tx: mpsc::Sender::<tungstenite::Message>,
  mut ser_req_shutdown_rx: watch::Receiver::<bool>
) -> Result<String, String> {
  // Start the tokio runtime for the server and launch the top-level server task.
  println!("Server launching runtime.");
  let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
  tokio_runtime.block_on(async {

    // Top-level tokio task
    // --------------------
    //
    let res = ser_thread_alive_tx.send(true);
    if res.is_err() { println!("Failed to set server alive."); return; }

    // Bind to websocket on localhost port 10202.
    let addr = "127.0.0.1:10202";
    let listener = TcpListener::bind(&addr).await;
    if listener.is_err() { println!("Failed to bind TcpListener."); return; }
    let listener = listener.unwrap();
    //.expect("Failed to bind to address")
    println!("Listening on: {}", addr);

    // Listen for connections until shutdown.
    // -----------------------------------
    //
    // Loop, responding to whichever future finishes first. (We break on a shutdown signal.)
    loop {
      let accept_conn = listener.accept();
      tokio::pin!(accept_conn);

      tokio::select! {
        // Valid connection. Launch task to handle the connection for its lifetime.
        Ok((stream, _)) = &mut accept_conn => {
          let peer = stream.peer_addr().expect("Connected streams should have a peer address");
          println!("[listen_for_websocket_connections] Peer address: {}", peer);

          // Each connection receives a reciever for messages to forward from the server, and a transmitter to forward client messages back to the server.
          let (ser_msg_broadcast_rx, cli_msg_store_tx) = (
            ser_msg_tx.subscribe(), cli_msg_tx.clone()
          );

          // Spawn a connection handler task, which will live for the duration of the connection.
          tokio::spawn(handle_connection(peer, stream, ser_msg_broadcast_rx, cli_msg_store_tx, ser_req_shutdown_rx.clone()));
        }

        // Receive an exit signal and shutdown.
        _ = ser_req_shutdown_rx.changed() => {
          if *ser_req_shutdown_rx.borrow() {
            println!("[listen_for_websocket_connections] Received shutdown signal.");
            break;
          }
        }
      } // tokio::select!
    } // loop

    // Shut down.
    println!("Server writing alive = false.");
    ser_thread_alive_tx.send(false).expect("Failed to set server thread alive to false.");
  });
  
  println!("Server shutting down.");
  Ok("Server shut-down successfully.".to_string())
}

async fn handle_connection(
  _peer: SocketAddr,
  stream: TcpStream,
  server_msg_rx: broadcast::Receiver<Vec<tungstenite::Message>>,
  client_msg_tx: mpsc::Sender<tungstenite::Message>,
  ser_req_shutdown_rx: watch::Receiver::<bool>
) {
  let addr = stream.peer_addr();
  if addr.is_err() {
    println!("[handle_connection] Error: Connected streams should have a peer address.");
    return;
  }
  let addr = addr.unwrap();

  let ws_stream = tokio_tungstenite::accept_async(stream)
    .await
    .expect("Error during the websocket handshake occurred");

  println!("[handle_connection] New websocket connection: {}", addr);
  
  // Split up the stream to a client reader and a client writer.
  let (ws_client_write, ws_client_read) = ws_stream.split();

  // Launch a task to handle sending messages from the server-side library consumer to the websocket client over ws_write.
  tokio::spawn(send_ws_client_messages(
    server_msg_rx, ws_client_write, ser_req_shutdown_rx.clone()
  ));

  // Launch a task to handle receiving message from the websocket client over ws_read and buffering them for the server-side library consumer to drain and handle later.
  tokio::spawn(recv_ws_client_messages(
    client_msg_tx, ws_client_read, ser_req_shutdown_rx
  ));

  // Archived: For debugging purposes, we can create a simple message forwarder for the lifetime of the connection (bouncing messages from the websocket client back to them).
  // let (write, read) = ws_stream.split();
  // read.forward(write).await.expect("Failed to forward message");
}

async fn send_ws_client_messages(
  mut server_msg_rx: broadcast::Receiver<Vec<tungstenite::Message>>,
  mut ws_client_write: SplitSink<WebSocketStream<TcpStream>, Message>,
  mut ser_req_shutdown_rx: watch::Receiver::<bool>
) {
  loop { tokio::select! {
    // Receive server messages and forward them to connected clients.
    recv_res = server_msg_rx.recv() => { match recv_res {
      Ok(msgs) => {
        for msg in msgs {
          let res = ws_client_write.feed(msg).await;
          if res.is_err() { println!("[tokio_server.rs] Failed to feed ws_client_write"); }
        }
        let res = ws_client_write.flush().await;
        if res.is_err() { println!("[tokio_server.rs] Failed to flush ws_client_write"); }
      }
      Err(err) => {
        println!("[send_ws_client_messages] Error sending msg to WS client: {:?}", err);
      }
    }}

    // Receive an exit signal and shutdown.
    _ = ser_req_shutdown_rx.changed() => {
      if *ser_req_shutdown_rx.borrow() {
        println!("[send_ws_client_messages] Received shutdown signal.");
        break;
      }
    }
  }}
}

async fn recv_ws_client_messages(
  client_msg_tx: mpsc::Sender<tungstenite::Message>,
  mut ws_client_read: SplitStream<WebSocketStream<TcpStream>>,
  mut ser_req_shutdown_rx: watch::Receiver::<bool>
) {
  // if let Some(Ok(foo)) = ws_client_read.next().await {
  //   foo.into_data()
  // }

  loop { tokio::select! {
    // Receive messages from connected clients and forward them to client message buffer.
    read_res = ws_client_read.next() => { match read_res {
      Some(Ok(msg)) => {
        let res = client_msg_tx.send(msg).await;
        if res.is_err() { println!("Failed to send client message to client msg buffer"); }
      }
      Some(Err(err)) => {
        println!("[send_ws_client_messages] Error sending msg to WS client: {:?}", err);
      }
      None => {
        println!("[send_ws_client_messages] None received from ws_client_read.next(), connection stream must be closed.");
        break;
      }
    }}

    // Receive an exit signal and shutdown.
    _ = ser_req_shutdown_rx.changed() => {
      if *ser_req_shutdown_rx.borrow() {
        println!("[send_ws_client_messages] Received shutdown signal.");
        break;
      }
    }
  }}
}
