// consumer_state.rs
//
// Internal handlers for managing static-scope (singleton) server state in a thread-safe manner.
//
// Static server state is guarded for thread-safe access using a blocking RwLock. This is definitely not optimal, and it'd probably be better to use tokio async locks and keep everything async, but I'm not sure what the best design for that is yet for a library receiving calls from the Python consumer thread. -Nick 2021-02-24

use std::{sync::{RwLock}};
use tokio::sync::{broadcast, mpsc, watch};

// ConsumerState
// -------------
//
/// Struct containing all the state we want to associate with the consumer of the library, mostly just communication channels to/from the tokio thread.
///
/// This struct is guarded by a RwLock, and is intended to protect against the consumer of the library calling in from more than one thread to receive messages from the client.
///
/// Generally we expect the RwLock not to be in contention, as we expect generally just one server-side consumer thread is going to be asking to e.g. drain client messages.
pub struct ConsumerState {
  /// Consumer thread(s) receiver for whether the Tokio server thread is alive.
  pub ser_thread_alive_rx: watch::Receiver<bool>,

  /// Consumer thread(s) clone of the server message transmitter, used to subscribe new receivers for any new connections.
  ///
  /// This is a clone of the Sender owned by the TokioState struct.
  pub ser_msg_tx: broadcast::Sender<Vec<tungstenite::Message>>,

  /// Consumer thread(s) receiver for messages from any connected clients. The server-side consumer should drain this receiver regularly.
  pub cli_msg_rx: mpsc::Receiver<tungstenite::Message>,

  /// Consumer thread(s) transmitter for requesting tokio to shut down.
  pub ser_req_shutdown_tx: watch::Sender<bool>,
}

// Lazy Static
// -----------
//
lazy_static! {
  /// Special storage associated with the library consumer, guarded by a RwLock. If the library consumer only ever calls into the library from a single thread, this lock won't hurt.
  static ref CONSUMER_STATE: RwLock<Option<ConsumerState>> = RwLock::new(None);

  /// Very coarse way of providing some quick error reporting to the consumer without panicking.
  static ref LAST_ERROR:     RwLock<Option<String>>        = RwLock::new(None);
}

// Error API
// ---------
//

/// Attempts to record the passed &str to the thread-safe LAST_ERROR storage. The method may fail silently if it can't get write access to LAST_ERROR, hence, "weakly". This is very likely to squelch errors if more than one propagates in short succession across more than one thread.
pub fn weakly_record_error(msg: String) {
  let last_err = LAST_ERROR.try_write();
  if last_err.is_err() { return; /* silently fail. */ }
  let mut last_err = last_err.unwrap();

  *last_err = Some(msg);
}

/// Returns the last error if possible. Returns None only if there is no last error.
pub fn try_get_last_error() -> Option<String> {
  let err_store = LAST_ERROR.read();
  if err_store.is_err() { /* Yo dawg */ return Some("Couldn't get last error.".to_string()); }

  // Peeeeel back the onion. RwLocks are like onions. RwLocks have *layers.*
  let err_store = err_store.as_deref();
  if err_store.is_err() { return Some("Couldn't get last error.".to_string()); }

  // If the error is None, great! Otherwise return a copy of the string content.
  let err_store = err_store.unwrap().as_ref();
  err_store.map_or(None, |s| Some(String::from(s)))
}


// State API
// ---------
//

pub fn read<F, T>(intent: &str, f: F) -> Option<T> where F: FnOnce(&ConsumerState) -> T {
  let read_guard = CONSUMER_STATE.read();
  if read_guard.is_err() {
    weakly_record_error(format!("Failed to get read access to consumer state. Intent was: {}", intent));
    return None;
  }
  let read_guard = read_guard.unwrap();

  let state = read_guard.as_ref();
  if state.is_none() {
    weakly_record_error(format!("Failed to get read access to consumer state, storage was empty. (Is the server running?) Intent was: {}", intent));
    return None;
  }
  let state = state.unwrap();

  Some(f(state))
}

pub fn write<F, T>(intent: &str, f: F) -> Option<T> where F: FnOnce(&mut ConsumerState) -> T {
  let write_guard = CONSUMER_STATE.write();
  if write_guard.is_err() {
    weakly_record_error(format!("Failed to get write access to consumer state. Intent was: {}", intent));
    return None;
  }
  let mut write_guard = write_guard.unwrap();

  let state = write_guard.as_mut();
  if state.is_none() {
    weakly_record_error(format!("Failed to get write access to consumer state, storage was empty. (Is the server running?) Intent was: {}", intent));
    return None;
  }
  let state = state.unwrap();

  Some(f(state))
}

pub fn set_consumer_state(consumer_state: ConsumerState) -> Result<(), ()> {
  let write_guard = CONSUMER_STATE.write();
  if write_guard.is_err() {
    weakly_record_error(format!("Failed to get write access to consumer state to set whole object."));
    return Err(());
  }
  let mut write_guard = write_guard.unwrap();

  (*write_guard) = Some(consumer_state);
  Ok(())
}
