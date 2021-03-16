
// Example usage of a consumer_state module
// ----------------------------------------
//
// (e.g. put these in api.rs)
//

fn _read_is_server_alive() {
  consumer_state::read("read server alive", |state| {
    state.ser_thread_alive_rx.recv()
  })
}

fn _send_msg(msg_bytes: Vec<u8>) {
  consumer_state::write("send msg", |state| {
    state.ser_msg_multi_tx.send(msg_bytes)
  })
}
