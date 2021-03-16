use tokio::sync::{broadcast, mpsc, oneshot, watch};

// TODO MIGRATE
pub mod state;
// pub mod consumer_state; // TODO MIGRATE
// TODO MIGRATE

mod tokio_server;

pub fn start() -> Result<(), ()> {
    // Create the broadcaster that will send messages provided by the server-side consumer to all connected websocket clients.
    // The broadcaster is stored statically and is accessed in API calls when the consumer wants to send messages through the server to any connected websocket clients.
    let (ser_msg_multi_tx, _) = broadcast::channel::<Vec<u8>>(16);

    // OH GOD IS THIS OK?
    let (cli_msg_store_multi_tx, cli_msg_store_single_rx) = mpsc::channel::<Vec<u8>>(16);

    // // // // Try to get a writeable lock on the server and initialize the server state.
    // // // {
    // // //   let server_state = state::try_get_writeable_server_state("Launch server and assign new server state");
    // // //   if server_state.is_none() { return Err(()); }
    // // //   let mut server_state = server_state.unwrap();
  
    // // //   // Create a fresh server state and start the server thread.
    // // //   let new_server_state = state::ServerState {
    // // //     is_shutdown_requested:    false,
    // // //     is_thread_alive:          false,
    // // //     ser_msg_multi_tx:         ser_msg_multi_tx,
    // // //     cli_msg_store_single_rx:  cli_msg_store_single_rx,
    // // //     cli_msg_store_multi_tx:   cli_msg_store_multi_tx
    // // //   };
    // // //   // // // let new_state = ServerState::new(thread_handle);
    // // //   *server_state = Some(new_server_state);

    // // //   // server_state dropped
    // // // }

    // // // // Launch the tokio thread.
    // // // let _thread_handle = thread::spawn(|| {
    // // //   server_thread_main()
    // // // });

    Ok(())
}


// TODO: TokioState doesn't have to exist, because all the necessary state can just be owned by the actual tokio thread!
// TODO: TokioState doesn't have to exist, because all the necessary state can just be owned by the actual tokio thread!
// TODO: TokioState doesn't have to exist, because all the necessary state can just be owned by the actual tokio thread!

// TokioState
// ----------
//
/// This struct is async-guarded, intended for source data to clone per-connection.
///
/// Each connection is expected to own its own data, so don't try to access this struct directly inside a connection handler. Otherwise the connections are going to fight for write access.
pub struct TokioState {
  /// Main tokio thread transmitter for whether the thread is alive.
  pub ser_thread_alive_tx: watch::Sender<bool>,

  /// Tokio thread(s) clone of the server message transmitter, used to subscribe new receivers for any new connections.
  ///
  /// This is a clone of the Sender owned by the ConsumerState struct.
  pub ser_msg_multi_tx: broadcast::Sender<Vec<u8>>,

  /// Tokio thread(s) transmitter (to clone per-connection) reporting back messages from any given connected client.
  pub cli_msg_store_multi_tx: mpsc::Sender<Vec<u8>>,

  /// Tokio thread(s) receiver for stopping tasks when shutdown is requested.
  pub ser_req_shutdown_rx: oneshot::Receiver::<()>
}
// TODO: TokioState doesn't have to exist, because all the necessary state can just be owned by the actual tokio thread!

