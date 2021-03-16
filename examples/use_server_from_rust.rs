use std::{thread::sleep, time::Duration};

fn main() {
  // Start the server and sleep for a bit.
  println!("[rs consumer test] Starting the server.");
  webviz_server_rs::start_server();
  sleep(Duration::from_millis(1000));

  // Check the server status and sleep again.
  let is_server_running = webviz_server_rs::is_server_running();
  if !is_server_running {
    println!("Unexpectedly, server failed to run.");
    return;
  }
  println!("[rs consumer test] Is the server running? {}", is_server_running);
  println!("Server will run for 10 seconds.");
  sleep(Duration::from_secs(10));

  // Request server shutdown and wait again.
  println!("[rs consumer test] Requesting server shutdown.");
  webviz_server_rs::shutdown_server();
  sleep(Duration::from_millis(1000));

  println!("[rs consumer test] Is the server running now? {}", webviz_server_rs::is_server_running());
  sleep(Duration::from_millis(1000));

  println!("[rs consumer test] End of main().");
}
