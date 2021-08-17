use std::{net::SocketAddr};
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use tokio::{net::{TcpListener, TcpStream}, sync::{broadcast, mpsc, watch}};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

/// Main thread loop for running the websocket server.
///
/// This function launches a tokio runtime to handle most server functions. The function will return after the tokio runtime exits.
pub fn main(
  port: u32,
  ser_thread_alive_tx: watch::Sender::<bool>,
  cli_conn_tokio_tx: mpsc::Sender<String>,
  ser_msg_tx: broadcast::Sender::<Vec<tokio_tungstenite::tungstenite::Message>>,
  cli_msg_tx: mpsc::Sender::<tokio_tungstenite::tungstenite::Message>,
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

    // Bind to websocket on localhost port 59994.
    let addr = format!("127.0.0.1:{}", port);
    println!("[quicksocket] Attempting to bind TcpListener at: {}", addr);
    let listener = TcpListener::bind(&addr).await;
    if listener.is_err() { println!("Failed to bind TcpListener. It's possible that port {} is already in use.", port); return; }
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
          println!("[tokio_server.rs] Peer address: {}", peer);
          let new_client_evt = peer.to_string();
          cli_conn_tokio_tx.send(new_client_evt).await.unwrap_or_else(|_| println!("[tokio_server.rs] Failed to report new client event to consumer."));

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
            println!("[tokio_server.rs] Received shutdown signal.");
            break;
          }
        }
      } // tokio::select!
    } // loop

    // Shut down.
    println!("[tokio_server.rs] Server writing alive = false.");
    ser_thread_alive_tx.send(false).unwrap_or_else(|_| println!("[tokio_server.rs] Failed to set server thread alive to false!"));
  });
  
  println!("[tokio_server.rs] Server tokio thread exiting.");
  Ok("Server shut-down successfully.".to_string())
}

async fn handle_connection(
  _peer: SocketAddr,
  stream: TcpStream,
  server_msg_rx: broadcast::Receiver<Vec<tokio_tungstenite::tungstenite::Message>>,
  client_msg_tx: mpsc::Sender<tokio_tungstenite::tungstenite::Message>,
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

  // Create a channel between the tasks to handle a client-initiated shutdown handshake.
  let (ws_client_req_shutdown_tx, ws_client_req_shutdown_rx) = watch::channel::<()>(());

  // Launch a task to handle sending messages from the server-side library consumer to the websocket client over ws_write.
  tokio::spawn(send_ws_client_messages(
    server_msg_rx, ws_client_write, ser_req_shutdown_rx.clone(), ws_client_req_shutdown_rx
  ));

  // Launch a task to handle receiving messages from the websocket client over ws_read and buffering them for the server-side library consumer to drain and handle later.
  tokio::spawn(recv_ws_client_messages(
    client_msg_tx, ws_client_read, ser_req_shutdown_rx, ws_client_req_shutdown_tx
  ));

  // Archived: For debugging purposes, we can create a simple message forwarder for the lifetime of the connection (bouncing messages from the websocket client back to them).
  // let (write, read) = ws_stream.split();
  // read.forward(write).await.expect("Failed to forward message");

  println!("[handle_connection] Websocket connection handled.");
}

async fn send_ws_client_messages(
  mut server_msg_rx: broadcast::Receiver<Vec<tokio_tungstenite::tungstenite::Message>>,
  mut ws_client_write: SplitSink<WebSocketStream<TcpStream>, Message>,
  mut ser_req_shutdown_rx: watch::Receiver::<bool>,
  mut ws_client_req_shutdown_rx: watch::Receiver::<()>
) {
  loop { tokio::select! {
    // Receive server messages and forward them to connected clients.
    recv_res = server_msg_rx.recv() => { match recv_res {
      Ok(msgs) => {
        for msg in msgs {
          let res = ws_client_write.feed(msg).await;
          if res.is_err() {
            println!("[send_ws_client_messages] Failed to feed ws_client_write. Assuming the connection has closed; terminating server forwarding task for this client.");
            break;
          }
        }
        let res = ws_client_write.flush().await;
        if res.is_err() {
          println!("[send_ws_client_messages] Failed to flush ws_client_write. Assuming the connection has closed; terminating server forwarding task for this client.");
          break;
        }
      }
      Err(err) => {
        println!("[send_ws_client_messages] Error sending msg to WS client: {:?}", err);
      }
    }}

    // Receive a shutdown signal from the client receiver task, indicating the client sent a shutdown handshake.
    _ = ws_client_req_shutdown_rx.changed() => {
      println!("[send_ws_client_messages] Received shutdown signal from the client receiver task; the client wants to disconnect. Resolving the shutdown handshake.");
      let res = ws_client_write.close().await;
      if let Err(err) = res {
        println!("[send_ws_client_messages] Error closing ws_client_write: {:?}", err);
      }
      break;
    }

    // Receive an exit signal and shutdown.
    _ = ser_req_shutdown_rx.changed() => {
      if *ser_req_shutdown_rx.borrow() {
        println!("[send_ws_client_messages] Received shutdown signal.");
        break;
      }
    }
  }}
  println!("[send_ws_client_messages] Client sender loop shutdown.")
}

async fn recv_ws_client_messages(
  client_msg_tx: mpsc::Sender<tokio_tungstenite::tungstenite::Message>,
  mut ws_client_read: SplitStream<WebSocketStream<TcpStream>>,
  mut ser_req_shutdown_rx: watch::Receiver::<bool>,
  ws_client_req_shutdown_tx: watch::Sender::<()>
) {
  loop { tokio::select! {
    // Receive messages from connected clients and forward them to client message buffer.
    read_res = ws_client_read.next() => { match read_res {
      Some(Ok(msg)) => {
        let res = client_msg_tx.send(msg).await;
        if res.is_err() { println!("[recv_ws_client_messages] Failed to send client message to client msg buffer"); }
      }
      Some(Err(err)) => {
        println!("[recv_ws_client_messages] Error receiving msg from WS client: {:?}", err);
      }
      None => {
        println!("[recv_ws_client_messages] None received from ws_client_read.next(), connection stream must be closed. Sending notification to the sender task.");

        // Send the shutdown signal to the sender-side task for this connection.
        let conn_shutdown_res = ws_client_req_shutdown_tx.send(());
        if let Err(err) = conn_shutdown_res {
          println!("[recv_ws_client_messages] Error sending a shutdown signal to the sender-side task for the closed connection: {:?}", err)
        }
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
  println!("[recv_ws_client_messages] Client receiver loop shutdown.")
}
