use std::thread;
use tokio::sync::{broadcast, mpsc, watch};

use consumer_state::weakly_record_error;

pub mod consumer_state;
mod tokio_server;

pub fn start() -> Result<(), ()> {
  // Server thread-alive channel.
  let (ser_thread_alive_tokio_tx, ser_thread_alive_consumer_rx) = {
    watch::channel::<bool>(false)
  };

  // Server message broadcast channel (consumer -> server -> client(s)).
  let (ser_msg_tokio_tx, _) = {
    broadcast::channel::<Vec<Vec<u8>>>(16)
  };
  // Both the consumer thread(s) and the tokio thread(s) will have their own copies of the transmitter.
  // The consumer thread uses its copy to send() messages. The tokio thread uses its copy to create per-connection receivers.
  let ser_msg_consumer_tx = ser_msg_tokio_tx.clone();

  // Client message channel.
  let (cli_msg_store_tokio_tx, cli_msg_store_consumer_rx) = {
    mpsc::channel::<Vec<u8>>(16)
  };

  // Shutdown channel.
  let (ser_req_shutdown_consumer_tx, ser_req_shutdown_tokio_rx) = {
    watch::channel::<bool>(false)
  };

  // Set the consumer thread(s) state object with all its relevant comms channels.
  let consumer_state_obj = consumer_state::ConsumerState {
    ser_thread_alive_rx: ser_thread_alive_consumer_rx,
    ser_msg_tx: ser_msg_consumer_tx,
    cli_msg_rx: cli_msg_store_consumer_rx,
    ser_req_shutdown_tx: ser_req_shutdown_consumer_tx
  };
  consumer_state::set_consumer_state(consumer_state_obj).unwrap_or_else(|_| {
    weakly_record_error("Failed to set consumer state! Server is not in a valid state.".to_string())
  });

  // Launch the tokio thread, passing ownership of all the tokio-side channels.
  // Launch the tokio thread.
  let _thread_handle = thread::spawn(|| tokio_server::main(
    ser_thread_alive_tokio_tx,
    ser_msg_tokio_tx,
    cli_msg_store_tokio_tx,
    ser_req_shutdown_tokio_rx
  ));

  Ok(())
}
