// consumer_state.rs
//
// Internal handlers for managing static-scope (singleton) server state in a thread-safe manner.
//
// Static server state is guarded for thread-safe access using a blocking RwLock. This is definitely not optimal, and it'd probably be better to use tokio async locks and keep everything async, but I'm not sure what the best design for that is yet for a library receiving calls from the Python consumer thread. -Nick 2021-02-24

use std::{sync::{RwLock}};
use tokio::sync::{broadcast, mpsc, watch};

type CS<T> = RwLock<Option<T>>;
type WsMessage = tokio_tungstenite::tungstenite::Message;

// Lazy Static
// -----------
//
// Various RwLock<Option<T>>s (here short-handed to CS, for Consumer State) guard against thread-unsafe access to consumer state -- consisting mostly of channels used to communicate with the tokio runtime thread. Channels as a whole are thread-safe, but the channel ends -- Senders, Receivers -- are expected to be used from just one thread at a time! Hence the RwLock, so the consumer can't get cheeky with the channels when invoking library functions from different threads.
//
// However, it's totally OK for the consumer to access *different* channel ends from different threads; e.g., to check for client messages on one thread while sending server messages from another.

lazy_static! {
  /// Consumer thread(s) receiver for whether the Tokio server thread is alive.
  pub static ref CS_SER_ALIVE_RX: CS<watch::Receiver<bool>> =
    RwLock::new(None);

  /// Consumer thread(s) receiver for events indicating newly-connected clients. The server-side consumer should drain this receiver regularly.
  pub static ref CS_CLI_CONN_RX: CS<mpsc::Receiver<String>> =
    RwLock::new(None);

  /// Consumer thread(s) clone of the server message transmitter, used to subscribe new receivers for any new connections.
  ///
  /// This is a clone of the Sender owned by the TokioState struct.
  pub static ref CS_SER_MSG_TX: CS<broadcast::Sender<Vec<WsMessage>>> =
    RwLock::new(None);

  /// Consumer thread(s) receiver for messages from any connected clients. The server-side consumer should drain this receiver regularly.
  pub static ref CS_CLI_MSG_RX: CS<mpsc::Receiver<WsMessage>> =
    RwLock::new(None);

  /// Consumer thread(s) transmitter for requesting tokio to shut down.
  pub static ref CS_SER_REQ_SHUTDOWN_TX: CS<watch::Sender<bool>> =
    RwLock::new(None);

  /// Very coarse way of providing some quick error reporting to the consumer without panicking.
  static ref LAST_ERROR: CS<String> = RwLock::new(None);
}

// Error API
// ---------
//
// This is probably not a very good error API. -Nick 2021-04-21

/// Attempts to record the passed &str to the thread-safe LAST_ERROR storage. The method may fail silently if it can't get write access to LAST_ERROR, hence, "weakly". This is very likely to clobber errors if more than one propagates in short succession across more than one thread.
pub fn weakly_record_error(msg: String) {
  let last_err = LAST_ERROR.try_write();
  if last_err.is_err() { return; /* silently fail. */ }
  let mut last_err = last_err.unwrap();

  *last_err = Some(msg);
}

/// Returns the last error if possible. Returns None only if there is no last error.
pub fn try_get_last_error() -> Option<String> {
  // Attempt to lock and unwrap.
  let err_store = LAST_ERROR.read();
  if err_store.is_err() { return Some("Couldn't get last error.".to_string()); }

  // Convert (possibly poisoned) lock into a Result.
  let err_store = err_store.as_deref();
  if err_store.is_err() { return Some("Couldn't get last error.".to_string()); }

  // If the error is None, great! Otherwise return a copy of the string content.
  let err_store = err_store.unwrap().as_ref();
  err_store.map_or(None, |s| Some(String::from(s)))
}

// State API
// ---------
//

/// Pass one of the "CS_" (consumer state) statics available in the consumer_state module and an operating function to do something with read access to that static (e.g. receive a message from a consumer channel).
pub fn read<T, U, R, F>(lazy_static_item: &R, f: F) -> Option<U>
where
  F: FnOnce(&T) -> U,
  R: std::ops::Deref<Target = RwLock<Option<T>>>
{
  let read_guard = lazy_static_item.read();
  if read_guard.is_err() {
    weakly_record_error(format!("Failed to get read access to {}.", std::any::type_name::<R>()));
    return None;
  }
  let read_guard = read_guard.unwrap();

  let item = read_guard.as_ref();
  if item.is_none() {
    weakly_record_error(format!("Failed to get read access to {}, as it hasn't been created yet. Is the server running?", std::any::type_name::<R>()));
    return None;
  }
  let item = item.unwrap();

  Some(f(item))
}

/// Pass one of the "CS_" (consumer state) statics available in the consumer_state module and an operating function to do something with mutable access to that static (e.g. send a message using a Sender).
pub fn mutate<T, U, R, F>(lazy_static_item: &R, f: F) -> Option<U>
where
  F: FnOnce(&mut T) -> U,
  R: std::ops::Deref<Target = RwLock<Option<T>>>
{
  let write_guard = lazy_static_item.write();
  if write_guard.is_err() {
    weakly_record_error(format!("Failed to get write access to {}.", std::any::type_name::<R>()));
    return None;
  }
  let mut write_guard = write_guard.unwrap();

  let state = write_guard.as_mut();
  if state.is_none() {
    weakly_record_error(format!("Failed to mutable ref for {}, as it hasn't been created yet. Is the server running?", std::any::type_name::<R>()));
    return None;
  }
  let state = state.unwrap();

  Some(f(state))
}


/// Pass one of the "CS_" (consumer state) statics available in the consumer_state module and a value of the inner type to set the RwLock<Option<T>> with Some<T>. This is used internally by the server::start() function to initialize consumer-side channels.
pub fn set_value<T, R>(lazy_static_item: &R, new_val: T) -> Result<(), ()>
where
  R: std::ops::Deref<Target = RwLock<Option<T>>>
{
  let write_guard = lazy_static_item.write();
  if write_guard.is_err() {
    weakly_record_error(format!("Failed to get write access to {} to set its value.", std::any::type_name::<R>()));
    return Err(());
  }
  let mut write_guard = write_guard.unwrap();

  (*write_guard) = Some(new_val);
  Ok(())
}
