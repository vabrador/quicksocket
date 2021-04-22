
# quicksocket

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Actions Status](https://github.com/nickjbenson/quicksocket/workflows/Build/badge.svg)](https://github.com/nickjbenson/quicksocket/actions)
[![PyPI](https://img.shields.io/pypi/v/quicksocket.svg?style=flat)](https://pypi.org/project/quicksocket/)
[![Crates.io](https://img.shields.io/crates/v/quicksocket.svg?style=flat)](https://crates.io/crates/quicksocket)

A simple WebSocket server built in Rust using [tokio](https://tokio.rs/), [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite), and [pyo3](https://github.com/PyO3/PyO3).

```sh
pip install quicksocket
```

Check out example-usage/quicksocket_with_types.py for an example of a wrapper WebsocketServer class around the quicksocket API and also provides type annotations for e.g. autocompletion in your IDE.

## Not stable, a bit verbose, and a bit messy!

The server listens for WebSocket connections on port `50808`, and currently this is not configurable. (see src/server/tokio_server.rs).

quicksocket's code is originally designed for use with Ultraleap's Web Visualizer project, and as such is intended for a console python visualizer server and leverages plain `println!`s for logging purposes.

At some point in the future it'll make sense to switch to some proper env logging system. Pardon the mess :)

# Building Locally

You'll need [Rust](https://rustup.rs/) and access to some python.exe of version 3.6 or greater.

You'll also need OpenSSL. See the Ubuntu section for installation details on Ubuntu. OpenSSL is slightly trickier for Windows. You can use Chocolatey or vcpkg, or download a binary distribution of OpenSSL. You may need to set `$Env:OPENSSL_DIR` (PowerShell syntax) to your installation directory for your Windows build session if using Chocolatey, or a binary install, or if vcpkg isn't activated for your session.

Once Rust is installed:
```sh
cargo build --release
```

This will output an .so or a .pyd depending on your OS into the `target/release` directory.

To build a wheel for your python/OS combo:
```sh
pip install maturin
maturin build
```

There's CI for Linux & Windows for Pythons 3.6 through 3.9, check out the Actions tab for now (proper release tags coming soon).

## Targeting builds to specific Python versions

If you're targeting a specific python version, or if you don't have python on your PATH, you can set `PYTHON_SYS_EXECUTABLE` to the python executable in your machine with that version. 

e.g., on Windows via Git Bash:
```sh
PYTHON_SYS_EXECUTABLE=/c/my/path/to/python.exe cargo build --release
```

## Ubuntu

Make sure you have libssl and libpython installed:
```sh
sudo apt-get install libssl-dev
sudo apt-get install libpython3-dev
```

If you encounter errors building `pyo3`, you may need to check whether it can find yor python and any related python dev dependencies: https://pyo3.rs/v0.10.1/building_and_distribution.html

## Todos:

#### Little utility HTML via github pages
Could expose this through Github Pages so it's easy for a new user to test their server usage:
https://github.com/nickjbenson/quicksocket/blob/main/archive/examples/wip01_basic_websocket_client.html
