// webviz-server
// =============
//
// A module for handling websocket connections through Rust. This module targets both Rust consumption (for testing purposes mainly) and compilation as a native Python module for consumption via Python.

#[macro_use]
extern crate lazy_static;

mod server;
mod api;

pub use api::*;
