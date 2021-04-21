// quicksocket
// =============
//
// A simple WebSocket server that is compiled via pyo3 to a native Python module. This module targets Python consumption primarily, but Rust consumption is theoretically also valid, and used for testing purposes.

#[macro_use]
extern crate lazy_static;

mod server;
mod api;

pub use api::*;
