// server_state.rs
//
// Internal handlers for managing static-scope (singleton) server state in a thread-safe manner.
//
// Static server state is guarded for thread-safe access using a blocking RwLock. This is definitely not optimal, and it'd probably be better to use tokio async locks and keep everything async, but I'm not sure what the best design for that is yet for a library receiving calls from the Python consumer thread. -Nick 2021-02-24

use std::{sync::{RwLock, RwLockWriteGuard}, thread::JoinHandle};
use tokio::sync::{broadcast, mpsc, oneshot, watch};

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
  pub ser_msg_multi_tx: broadcast::Sender<Vec<u8>>,

  /// Consumer thread(s) receiver for messages from any connected clients. The server-side consumer should drain this receiver regularly.
  pub cli_msg_store_single_rx: mpsc::Receiver<Vec<u8>>,

  /// Consumer thread(s) transmitter for requesting tokio to shut down.
  pub ser_req_shutdown_tx: oneshot::Sender<()>,
}

// Lazy Static
// -----------
//
lazy_static! {
  /// Special storage associated with the library consumer, guarded by a RwLock. If the library consumer only ever calls into the library from a single thread, this lock won't hurt.
  static ref CONSUMER_STATE: RwLock<Option<ConsumerState>> = RwLock::new(None);

  /// Very coarse way of providing some quick error reporting to the consumer without panicking.
  static ref LAST_ERROR:     RwLock<Option<String>>             = RwLock::new(None);
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

// Consumer State API
// ------------------

// TODO IMPLEMENTME








// archive

// // // // Server State API
// // // // ----------------
// // // //

// // // /// Attempts to read server state. May fail if this thread fails to lock the server state, which will cause the function to return None.
// // // pub fn try_read_server_state<F, T>(intent: &str, read_fn: F) -> Option<T> where F: FnOnce(&ServerState) -> T {
// // //   let server_store = SERVER_STATE.read();
// // //   if server_store.is_err() {
// // //     weakly_record_error(format!("Failed to retrieve readable server state. Intent: {}", intent));
// // //     return None;
// // //   }
// // //   let server_state = server_store.unwrap();
// // //   if server_state.is_none() { return None; }

// // //   Some(read_fn(server_state.as_ref().unwrap()))
// // // }

// // // /// Attempts to give writeable server state within a closure. May fail if this thread fails to lock the server state, which will cause the function to return None.
// // // pub fn try_write_server_state<F, T>(intent: &str, write_fn: F) -> Option<T> where F: FnOnce(&mut ServerState) -> T {
// // //   let server_store = SERVER_STATE.write();
// // //   if server_store.is_err() {
// // //     weakly_record_error(format!("Failed to retrieve writeable server state. Intent: {}", intent));
// // //     return None;
// // //   }
// // //   let mut server_state = server_store.unwrap();
// // //   if server_state.is_none() { return None; }

// // //   Some(write_fn(server_state.as_mut().unwrap()))
// // // }

// // // /// Attempts to retreive a writeable server state.
// // // /// 
// // // /// The server initiailization code uses this to initialize the server state struct (where it starts as None), rather than just modify an existing server state struct.
// // // pub fn try_get_writeable_server_state(intent: &str) -> Option<RwLockWriteGuard<Option<ServerState>>> {
// // //   let server_store = SERVER_STATE.write();
// // //   if server_store.is_err() {
// // //       weakly_record_error(format!("Failed to get writeable server state. Intent was: {}", intent));
// // //       println!("Failed to get writeable server state with intent: {}", intent);
// // //       return None;
// // //   }

// // //   Some(server_store.unwrap())
// // // }

// // // /// Writes whether the server has been requested to shut down.
// // // pub fn request_server_shutdown() {
// // //   try_get_writeable_server_state("Request server shutdown")
// // //   .and_then(|mut server_state| {
// // //       (*server_state).as_mut().unwrap().is_shutdown_requested = true;
// // //       Some(())
// // //   });
// // // }

// // // // Server Thread API
// // // // -----------------
// // // //

// // // pub fn try_read_server_handle<F, T>(intent: &str, read_fn: F) -> Option<T> where F: FnOnce(&JoinHandle<SrvRes>) -> T {
// // //   let handle_store = SERVER_THREAD.read();
// // //   if handle_store.is_err() {
// // //     weakly_record_error(format!("Failed to retrieve readable server thread JoinHandle. Intent: {}", intent));
// // //     return None;
// // //   }
// // //   let handle_state = handle_store.unwrap();
// // //   if handle_state.is_none() { return None; }

// // //   Some(read_fn(handle_state.as_ref().unwrap()))
// // // }
