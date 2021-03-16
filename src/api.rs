// api.rs
// ======
//
// Primary Python module and Rust-lib public API for the webviz-server library.

use pyo3::{prelude::*, wrap_pyfunction};

use crate::server::{self, state::weakly_record_error};
use server::state;

/// Starts the websocket server.
#[pyfunction]
pub fn start_server() -> bool {
    // For now, start_server can only be called if the server is not already running.
    if is_server_running() {
      state::weakly_record_error("Server is already running, can't invoke start_server().".to_string());
      return false;
    }

    let server_started = server::start().is_ok();
    if !server_started { return false; }

    println!("Server started.");
    return true;
}

/// Gets whether the server is running.
#[pyfunction]
pub fn is_server_running() -> bool {
    // e.g...
    // consumer_state::read("read server alive", |state| {
    //     state.ser_thread_alive_rx.recv()
    // })

    let is_running = state::try_read_server_state("Check server running status", |server_state| { server_state.is_thread_alive })
    .unwrap_or(false);
    
    println!("Returning is_server_running {}", is_running);
    is_running
}

/// Requests that the websocket server shut down. The server will not shut down immediately but will stop serving as soon as e.g. it processes the shutdown request and any existing network requests are resolved.
#[pyfunction]
pub fn shutdown_server() {
    // state::request_server_shutdown();
    state::try_write_server_state(
        "Request server shutdown",
        |state| { state.is_shutdown_requested = true }
    );
}

/// Gets whether a server shutdown has been requested. If true, the server will shut down soon.
#[pyfunction]
pub fn is_shutdown_requested() -> bool {
    state::try_read_server_state("Check server shutdown-requested status", |server_state| {
        server_state.is_shutdown_requested
    }).unwrap_or(false)
}

/// Returns a string describing the nature of the last error the server encountered. No error has been detected if this function returns None.
#[pyfunction]
pub fn get_last_error_string() -> Option<String> {
    state::try_get_last_error()
}

/// Attempts to send messages bytes to the tokio runtime. Will return false if there are not currently any active subscribers (websocket clients), indicating no data was sent. False may also be returned if there was an error trying to access the broadcast channel in the first place (i.e. thread contention to access it). A return value of true does not guarantee all websocket clients received the message, as the tokio tasks for forwarding the messages to the clients must be able to receive the broadcast messages to forward them, which is subject to thread/task contention.
#[pyfunction]
pub fn try_send_message_bytes(bytes: Vec<u8>) -> bool {
    let send_res = state::try_read_server_state("Send message bytes", |state| {
        // Send!
        state.ser_msg_multi_tx.send(bytes)
    });
    // Check whether, and precisely how, we failed to send.
    if send_res.is_none() || send_res.unwrap().is_err() {
        let mut details = "Error reading server state for transmitter".to_string();
        if send_res.is_some() {
            details = format!("{:?}", send_res.unwrap().err());
        }
        weakly_record_error(format!("Failed to send message. Details: {}", details));
        return false;
    }
    true
}
// pub fn send_message(msg: HashMap<String, HashMap<String, &PyAny>>) -> bool {

/// Drains all messages pending from all clients and returns them as a list[bytes]. Note that clients are not distinguished, so clients will have to self-identify in their messages, or the library will need to change to return messages per-client or bundled with client connection info.
#[pyfunction]
pub fn drain_client_message_bytes() -> Vec<Vec<u8>> {
    let drained_messages = state::try_read_server_state("Drain client messages", |state| {
        let mut messages = vec![];

        // Apparently there's an issue with try_recv() where messages may not be immediately available once submitted to the channel (they may be subject to a slight delay).
        // Details: https://github.com/tokio-rs/tokio/issues/3350
        // TODO: May look into using 'flume', with some tokio-based sync primitive on the tokio task side.
        while let Some(Some(cli_msg)) = state.cli_msg_store_single_rx.recv().now_or_never() {
            messages.push(cli_msg);
        }

        messages
    });
    if drained_messages.is_none() {
        return vec![];
    }
    let drained_messages = drained_messages.unwrap();

    drained_messages
}

/// The keras-hannd web visualizer websocket server as a native Python module, authored in Rust.
#[pymodule]
fn webviz_server_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server,               m)?)?;
    m.add_function(wrap_pyfunction!(is_server_running,          m)?)?;
    m.add_function(wrap_pyfunction!(shutdown_server,            m)?)?;
    m.add_function(wrap_pyfunction!(is_shutdown_requested,      m)?)?;
    m.add_function(wrap_pyfunction!(get_last_error_string,      m)?)?;
    m.add_function(wrap_pyfunction!(try_send_message_bytes,     m)?)?;
    m.add_function(wrap_pyfunction!(drain_client_message_bytes, m)?)?;

    Ok(())
}
