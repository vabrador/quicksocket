use std::{net::SocketAddr, time::Duration};
use futures_util::{StreamExt, stream::{SplitSink, SplitStream}};
use tokio::{net::{TcpListener, TcpStream}, sync::{broadcast, mpsc, oneshot, watch}};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

/// Main thread loop for running the websocket server.
///
/// This function launches a tokio runtime to handle most server functions. The function will return after the tokio runtime exits. See server_async_main() for more details on the tasks the server actually performs.
pub fn server_thread_main() -> Result<String, String> {
  // Start the tokio runtime for the server and launch the top-level server task.
  println!("Server launching runtime.");
  let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
  tokio_runtime.block_on(server_async_main());
  
  println!("Server shutting down.");
  Ok("Server shut-down successfully.".to_string())
}

/// The async function that handles the actual server work, run by the tokio runtime from the main server_thread() function.
///
/// Through this function, the server handles websocket requests and periodically checks for external signals (such as a shutdown signal).
async fn server_async_main() {
  // Write that the server is alive.
  println!("Server writing alive = true.");
  state::try_write_server_state("Write server thread alive = true", |state| {
    state.is_thread_alive = true;
  });

  // Bind to websocket on localhost port 10202.
  let addr = "127.0.0.1:10202";
  let listener = TcpListener::bind(&addr).await.expect("Failed to bind to address");
  println!("Listening on: {}", addr);

  // Single-producer, many-consumer channel.
  let (shutdown_tx, shutdown_rx) = watch::channel::<bool>(false);

  // Launch tasks.
  // -------------
  //
  let websocket_task = tokio::spawn(
    listen_for_websocket_connections(
      listener,
      shutdown_rx.clone()
    )
  );
  let shutdown_task = tokio::spawn(
    server_periodically_check_shutdown_signal(shutdown_tx)
  );
  
  // Join tasks for shutdown.
  // ------------------------
  //
  if shutdown_task.await.is_err() {
    println!("[server_async_main] shutdown_task errored while awaiting :(");
  }
  if websocket_task.await.is_err() {
    println!("[server_async_main] websocket_task errored while awaiting :(");
  }

  println!("Server writing alive = false.");
  state::try_write_server_state("Write server thread alive = false", |state| {
    state.is_thread_alive = false;
  });
}

async fn listen_for_websocket_connections(
  listener: TcpListener,
  mut exit_rx: watch::Receiver<bool>
) {
  let accept_conn = listener.accept();
  tokio::pin!(accept_conn);

  loop {
    // Await either a valid connection or the shutdown signal, whichever occurs first.
    tokio::select! {
      // Valid connection. Launch task to handle the connection for its lifetime.
      Ok((stream, _)) = &mut accept_conn => {
        let peer = stream.peer_addr().expect("Connected streams should have a peer address");
        println!("[listen_for_websocket_connections] Peer address: {}", peer);

        // Each connection receives a reciever for messages to forward from the server, and a transmitter to forward client messages back to the server.
        let opt_chs = state::try_read_server_state("Handle connection; get server/client rx/tx", |state| {
          (state.ser_msg_multi_tx.subscribe(), state.cli_msg_store_multi_tx.clone())
        });
        if opt_chs.is_none() { println!("[ERROR] Server state is poisoned! Can't handle comms."); }
        let (ser_msg_broadcast_rx, cli_msg_store_tx) = opt_chs.unwrap();

        // Spawn the connection handler task, living for the duration of the connection.
        tokio::spawn(handle_connection(peer, stream, ser_msg_broadcast_rx, cli_msg_store_tx));
      }

      // Receive an exit signal and shutdown.
      _ = exit_rx.changed() => {
        if *exit_rx.borrow() {
          println!("[listen_for_websocket_connections] Received shutdown signal.");
          break;
        }
      }
    } // tokio::select!
  } // loop
}

async fn handle_connection(
  _peer: SocketAddr,
  stream: TcpStream,
  server_msg_rx: broadcast::Receiver<Vec<u8>>,
  client_msg_tx: mpsc::Sender<Vec<u8>>
) {
  let addr = stream.peer_addr().expect("Connected streams should have a peer address");
  // println!("[Accept connection] Peer address: {}", addr);

  let ws_stream = tokio_tungstenite::accept_async(stream)
    .await
    .expect("Error during the websocket handshake occurred");

  println!("[Accept connection] New websocket connection: {}", addr);
  
  // Split up the stream to a client reader and a client writer.
  let (ws_client_write, ws_client_read) = ws_stream.split();

  // Launch a task to handle sending messages from the server-side library consumer to the websocket client over ws_write.
  tokio::spawn(send_ws_client_messages(server_msg_rx, ws_client_write));

  // Launch a task to handle receiving message from the websocket client over ws_read and buffering them for the server-side library consumer to drain and handle later.
  tokio::spawn(recv_ws_client_messages(client_msg_tx, ws_client_read));

  // Archived: For debugging purposes, this configuration would create a simple message forwarder for the lifetime of the connection (bouncing messages from the websocket client back to them).
  // let (write, read) = ws_stream.split();
  // read.forward(write).await.expect("Failed to forward message");
}

async fn send_ws_client_messages(server_msg_rx: broadcast::Receiver<Vec<u8>>, ws_client_write: SplitSink<WebSocketStream<TcpStream>, Message>) {
  // TODO: IMPLEMENTME
  compile_error!("NYI");
}

async fn recv_ws_client_messages(client_msg_tx: mpsc::Sender<Vec<u8>>, ws_client_read: SplitStream<WebSocketStream<TcpStream>>) {
  // TODO: IMPLEMENTME
  compile_error!("NYI");
}

// // // async fn listen_for_consumer_messages(mut msg_rx: mpsc::Receiver<Vec<u8>>, mut exit_rx: watch::Receiver<bool>) {
// // //   loop {
// // //     tokio::select! {
// // //       maybe_msg = msg_rx.recv() => {
// // //         // Launch the task to handle the message.
// // //         maybe_msg.map(|msg| tokio::spawn(handle_msg(msg)));
// // //       }
// // //       _ = exit_rx.changed() => {
// // //         if *exit_rx.borrow() {
// // //           println!("[listen_for_consumer_messages] Received shutdown signal.");
// // //         }
// // //       }
// // //     }
// // //   }
// // // }

// // // async fn handle_msg(msg_bytes: Vec<u8>) {
// // //   println!("Received message of {} bytes in length. TODO: Send the message to the websocket client!", msg_bytes.len())
// // // }

async fn server_periodically_check_shutdown_signal(shutdown_tx: watch::Sender<bool>) {
  loop {
      tokio::time::sleep(Duration::from_millis(100)).await;

      if is_shutdown_requested() {
        // Send shutdown signal to other looping tasks in the runtime.
        let res = shutdown_tx.send(true);
        if res.is_err() {
          println!("[server_periodically_check_shutdown_signal] Error reporting shutdown signal through shutdown_tx! {:?}", res.err());
        }

        // Break and exit.
        break;
      }
  }
}
