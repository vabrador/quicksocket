use std::thread;
use tokio::sync::{broadcast, mpsc, watch};

pub mod consumer_state;
mod tokio_server;

pub fn start(port: u32) -> Result<(), ()> {
  // Server thread-alive channel.
  let (ser_thread_alive_tokio_tx, ser_alive_consumer_rx) = {
    watch::channel::<bool>(false)
  };

  // Client connection event channel.
  let (cli_conn_tokio_tx, cli_conn_consumer_rx) = {
    mpsc::channel::<String>(16)
  };

  // Server message broadcast channel (consumer -> server -> client(s)).
  let (ser_msg_tokio_tx, _) = {
    broadcast::channel::<Vec<tokio_tungstenite::tungstenite::Message>>(16)
  };
  // Both the consumer thread(s) and the tokio thread(s) will have their own copies of the transmitter. The consumer thread uses its copy to send() messages. The tokio thread uses its copy to create per-connection receivers.
  let ser_msg_consumer_tx = ser_msg_tokio_tx.clone();

  // Client message channel.
  let (cli_msg_store_tokio_tx, cli_msg_store_consumer_rx) = {
    mpsc::channel::<tokio_tungstenite::tungstenite::Message>(16)
  };

  // Shutdown channel.
  let (ser_req_shutdown_consumer_tx, ser_req_shutdown_tokio_rx) = {
    watch::channel::<bool>(false)
  };

  // Set the consumer thread statics with all its relevant comms channels.
  use consumer_state as cs;
  cs::set_value(&cs::CS_SER_ALIVE_RX, ser_alive_consumer_rx)
    .expect("Failed to set consumer state channel!");
  cs::set_value(&cs::CS_CLI_CONN_RX, cli_conn_consumer_rx)
    .expect("Failed to set consumer state channel!");
  cs::set_value(&cs::CS_SER_MSG_TX, ser_msg_consumer_tx)
    .expect("Failed to set consumer state channel!");
  cs::set_value(&cs::CS_CLI_MSG_RX, cli_msg_store_consumer_rx)
    .expect("Failed to set consumer state channel!");
  cs::set_value(&cs::CS_SER_REQ_SHUTDOWN_TX, ser_req_shutdown_consumer_tx)
    .expect("Failed to set consumer state channel!");

  // Launch the tokio thread, passing ownership of all the tokio-side channels.
  // Launch the tokio thread.
  let _thread_handle = thread::spawn(move || tokio_server::main(
    port,
    ser_thread_alive_tokio_tx,
    cli_conn_tokio_tx,
    ser_msg_tokio_tx,
    cli_msg_store_tokio_tx,
    ser_req_shutdown_tokio_rx
  ));

  Ok(())
}
