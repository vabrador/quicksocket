
# quicksocket

!!! This repository is archived as of 2025-07-29. Use at your own risk.

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Apache v2 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Actions Status](https://github.com/nickjbenson/quicksocket/workflows/Build/badge.svg)](https://github.com/nickjbenson/quicksocket/actions)
[![PyPI](https://img.shields.io/pypi/v/quicksocket.svg?style=flat)](https://pypi.org/project/quicksocket/)
[![Crates.io](https://img.shields.io/crates/v/quicksocket.svg?style=flat)](https://crates.io/crates/quicksocket)

A simple WebSocket server built in Rust using [tokio](https://tokio.rs/), [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite), and [pyo3](https://github.com/PyO3/PyO3). Supports Windows, macOS, and Linux.

More stable as of 1.0.0, but still feature-light. You only get one server (need to add a *proper* Server class), and the features are pretty minimal.

```sh
pip install quicksocket
```
## Quick Start for Quicksocket

`quicksocket.server.Server` provides type annotations for the otherwise static methods available directly from the `quicksocket` module.

```python
import quicksocket

# Start the server on whatever port you want.
server = quicksocket.server.Server()
server.start(port=59994)

# You have to poll the server, which runs on a native Rust thread.
# 
# No need for `asyncio` here! Do it however you want.
# But you'll need some sort of loop for this.
new_clients = server.drain_new_client_events()
cli_msgs = server.drain_client_messages()

# Send messages in batches for better efficiency. Often, python's "threading" is a performance bottleneck.
message = "Hello, world!"
another_message = "Yes, hello!"
server.send_messages([message, another_message])

# Check if the server is running.
is_server_running = server.is_running()

# Stop the server.
server.stop()
```

## A bit verbose, and still stabilizing.

As of 1.0 the initial connection port is configurable, just pass the port to the `start` method.

Quicksocket's code is originally designed for use with Ultraleap's Web Visualizer project, and as such is intended for a console python visualizer server and leverages plain `println!`s for logging purposes. Coming "soon": Proper env logging.

# Building

You'll need [Rust](https://rustup.rs/) and access to some python.exe of version 3.6 or greater.

You'll also need OpenSSL. See the Ubuntu section for installation details on Ubuntu. OpenSSL is slightly trickier for Windows. You can use Chocolatey or vcpkg, or download a binary distribution of OpenSSL. You may need to set `$Env:OPENSSL_DIR` (PowerShell syntax) to your installation directory for your Windows build session if using Chocolatey, or a binary install, or if vcpkg isn't activated for your session. macOS should have it by default.

To build a wheel for your python/OS combo:
```sh
pip install maturin
maturin build
```

If you just want a raw `.pyd` or similar python native module file for your OS, you can just build with `cargo`:
```sh
cargo build --release
```

There's CI for Windows, macOS, and Linux for Pythons 3.6 through 3.9. Check out the Actions tab.

## Ubuntu

Make sure you have libssl and libpython installed:
```sh
sudo apt-get install libssl-dev
sudo apt-get install libpython3-dev
```

If you encounter errors building `pyo3`, you may need to check whether it can find your python and any related python dev dependencies: https://pyo3.rs/v0.10.1/building_and_distribution.html

# Older Build Instructions

## Targeting builds to specific Python versions

This only pertains to building via `cargo`. `maturin` is probably more reliable.

If you're targeting a specific python version, or if you don't have python on your PATH, you can set `PYTHON_SYS_EXECUTABLE` to the python executable in your machine with that version. 

e.g., on Windows via Git Bash:
```sh
PYTHON_SYS_EXECUTABLE=/c/my/path/to/python.exe cargo build --release
```

## Todos:

#### A utility HTML via github pages
Could expose this through Github Pages so it's easy for a new user to test their server usage...
https://github.com/nickjbenson/quicksocket/blob/main/archive/examples/wip01_basic_websocket_client.html
