
use std::{convert::Infallible, net::SocketAddr};
use hyper::{self, Body, Request, Response};
use tokio;

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
  Ok(Response::new("Hello, World".into()))
}

/// We don't use `#[tokio::main]` because we want to easily be able to transfer this executable-based example into a library-based one (that won't *have* a main function).
fn main() {
  let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
  tokio_runtime.block_on(async_main())
}

/// The "actual" main function, which is async and run by tokio.
async fn async_main() {
  let addr = "127.0.0.1:8080".parse::<SocketAddr>().expect("Error parsing address.");

  // A `Service` is needed for every connection, so this creates one from our `hello_world` function.
  let make_svc = hyper::service::make_service_fn(|_conn| async {
      // service_fn converts our function into a `Service`
      Ok::<_, Infallible>(hyper::service::service_fn(hello_world))
  });

  let server = hyper::Server::bind(&addr).serve(make_svc);

  // Run this server for... forever!
  if let Err(e) = server.await {
      eprintln!("server error: {}", e);
  }
}
