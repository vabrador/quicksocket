// api.rs
// ======
//
// Primary Python module and Rust-lib public API.

use futures_util::FutureExt;
use pyo3::{prelude::*, wrap_pyfunction};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::server::{self, consumer_state::{self}};
use consumer_state as cs;

/// Starts the websocket server.
#[pyfunction]
pub fn start_server(port: u32) -> bool {
    // For now, start_server can only be called if the server is not already running.
    if is_server_running() {
      consumer_state::weakly_record_error("Server is already running, can't invoke start_server().".to_string());
      return false;
    }

    let server_started = server::start(port).is_ok();
    if !server_started { return false; }

    println!("Server started.");
    return true;
}

/// Gets whether the server is running.
#[pyfunction]
pub fn is_server_running() -> bool {
    cs::read(&cs::CS_SER_ALIVE_RX, |rx| {
        println!("Returning server alive: {}", *rx.borrow());
        *rx.borrow()
    }).unwrap_or_else(|| {
        println!("Failed to get server alive!");
        false
    })
}

/// Requests that the websocket server shut down. The server will not shut down immediately but will stop serving as soon as e.g. it processes the shutdown request and any existing network requests are resolved.
#[pyfunction]
pub fn shutdown_server() {
    let res = cs::mutate(&cs::CS_SER_REQ_SHUTDOWN_TX, |tx| tx.send(true));
    if res.is_none() {
        println!("[api.rs] Warning! Failed to send shutdown request.");
    }
}

/// Returns a string describing the nature of the last error the server encountered. No error has been detected if this function returns None.
#[pyfunction]
pub fn get_last_error_string() -> Option<String> {
    consumer_state::try_get_last_error()
}

/// Retrieves a List (Rust: Vec<String>) of all new client connection events that have occurred since this function was last called.
#[pyfunction]
pub fn drain_new_client_events(py: Python) -> Vec<String> {
    py.allow_threads(|| {
        let drained_new_cli_evts = cs::mutate(&cs::CS_CLI_CONN_RX, |rx| {
            let mut new_cli_evts = vec![];
            while let Some(Some(new_cli)) = rx.recv().now_or_never() {
                new_cli_evts.push(new_cli);
            }
            new_cli_evts
        });
        if drained_new_cli_evts.is_none() {
            return vec![]
        }
        let drained_new_cli_evts = drained_new_cli_evts.unwrap();
        
        drained_new_cli_evts
    })
}

/// Valid message payloads in the list of messages to provide to try_send_message consist of strings (text messages) and bytes (binary messages).
///
/// Passing any other type within the list of objects will raise an exception.
#[derive(FromPyObject)]
pub enum MessagePayload {
    #[pyo3(transparent, annotation = "str")]
    Text(String),
    #[pyo3(transparent, annotation = "bytes")]
    Binary(Vec<u8>)
}
impl IntoPy<PyObject> for MessagePayload {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            MessagePayload::Text(text) => {
                pyo3::types::PyUnicode::new(py, text.as_str()).into()
            }
            MessagePayload::Binary(bytes) => {
                pyo3::types::PyBytes::new(py, &bytes).into()
            }
        }
    }
}

/// Send messages to all connected clients. The socket stream is flushed after buffering each message in the argument List, so it's better to call this once per 'update,' rather than calling this method multiple times if multiple messages are all available to be sent.
///
/// The List may contain strings or bytes.
///
/// Will return false if there are not currently any active subscribers (websocket clients), indicating no data was sent. False may also be returned if there was an error trying to access the broadcast channel in the first place (i.e. thread contention to access it).
///
/// A return value of true does not guarantee all websocket clients received the message, as the tokio tasks for forwarding the messages to the clients must be able to receive the broadcast messages to forward them, which is subject to thread/task contention.
#[pyfunction]
pub fn try_send_messages(py: Python, messages: Vec<MessagePayload>) -> PyResult<()> {
    py.allow_threads(|| {
        // Create a Vec<WsMessage> out of the Vec<MessagePayload> so the backend is just working with the tungstenite WebSocket lib types.
        let messages: Vec<WsMessage> = messages.into_iter().map(|msg| { match msg {
            MessagePayload::Text(text)   => { WsMessage::Text(text) }
            MessagePayload::Binary(bytes) => { WsMessage::Binary(bytes) }
        }}).collect();
    
        let send_res = cs::read(&cs::CS_SER_MSG_TX, |tx| {
            // Send!
            tx.send(messages)
        });
        // Check whether, and precisely how, we failed to send.
        if send_res.is_none() || send_res.as_ref().unwrap().is_err() {
            let details = "Error reading server state for transmitter".to_string();
            // if send_res.is_some() {
            //     details = format!("{:?}", send_res.unwrap().err());return Err(pyo3::exceptions::PyBaseException::new_err(format!("Failed to send message. Details: {}", details)));
            // }
            // For now, only return an error if the send fails unrelated to the number of receivers, because we simply expect the message to go nowhere if there are no connected clients.

            if send_res.is_none() {
                return Err(pyo3::exceptions::PyBaseException::new_err(format!("Failed to send message. Details: {}", details)));
            }
    
            // weakly_record_error(format!("Failed to send message. Details: {}", details));
            // return Err(pyo3::exceptions::PyBaseException::new_err(format!("Failed to send message. Details: {}", details)));
        }
    
        Ok(())
    })
}

/// Drains all messages pending from all clients and returns them as a list[bytes]. Note that clients are not distinguished, so clients will have to self-identify in their messages, or the library will need to change to return messages per-client or bundled with client connection info.
#[pyfunction]
pub fn drain_client_messages(py: Python) -> Vec<MessagePayload> {
    py.allow_threads(|| {
        let drained_messages = cs::mutate(&cs::CS_CLI_MSG_RX, |rx| {
            let mut messages = vec![];
    
            // Apparently there's an issue with try_recv() where messages may not be immediately available once submitted to the channel (they may be subject to a slight delay).
            // Details: https://github.com/tokio-rs/tokio/issues/3350
            // TODO: May look into using flume, with some tokio-based sync primitive on the tokio task side.
            while let Some(Some(cli_msg)) = rx.recv().now_or_never() {
                // Convert the message into the python-convertible MessagePayload type.
                // For now, we ignore the ping/pong and Close websocket messages.
                let converted_msg = match cli_msg {
                    WsMessage::Text(text)    => { Some(MessagePayload::Text(text)) }
                    WsMessage::Binary(bytes) => { Some(MessagePayload::Binary(bytes)) }
                    WsMessage::Ping(_)       => { None }
                    WsMessage::Pong(_)       => { None }
                    WsMessage::Close(_)      => { None }
                };
                if converted_msg.is_some() { messages.push(converted_msg.unwrap()); }
            }
    
            messages
        });
        if drained_messages.is_none() {
            return vec![];
        }
        let drained_messages = drained_messages.unwrap();
    
        drained_messages
    })
}

/// Defines the actual python module for pyo3 to generate.
#[pymodule]
fn quicksocket(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server,               m)?)?;
    m.add_function(wrap_pyfunction!(is_server_running,          m)?)?;
    m.add_function(wrap_pyfunction!(shutdown_server,            m)?)?;
    m.add_function(wrap_pyfunction!(get_last_error_string,      m)?)?;
    m.add_function(wrap_pyfunction!(drain_new_client_events,    m)?)?;
    m.add_function(wrap_pyfunction!(try_send_messages,          m)?)?;
    m.add_function(wrap_pyfunction!(drain_client_messages,      m)?)?;

    Ok(())
}
