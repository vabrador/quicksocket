
from asyncio.tasks import gather
import time

import quicksocket.server

import asyncio    # Dependency for use with `websockets`.
import websockets # A test websocket library to use as a client.

# The Tempest, Act 4, Scene 1.
TEMPEST_A4S1_CLIENT_CALL     = "We are such stuff as dreams are made on"
TEMPEST_A4S1_SERVER_RESPONSE = "And our little life is rounded with a sleep."

async def client_handshake(port: int):
  '''Send the 'secret' test passphrase to the server.'''
  print("[test_handshake] [client_handshake] Starting.")

  uri = "ws://localhost:" + str(port)
  async with websockets.connect(uri) as websocket:

    print("[test_handshake] [client_handshake] Sending client message.")
    await websocket.send(TEMPEST_A4S1_CLIENT_CALL)

    print("[test_handshake] [client_handshake] Awaiting response.")
    response = await websocket.recv()

    print("[test_handshake] [client_handshake] Got response: {}".format(TEMPEST_A4S1_SERVER_RESPONSE))
    assert(response == TEMPEST_A4S1_SERVER_RESPONSE)
    await asyncio.sleep(0.200)

    print("[test_handshake] [client_handshake] Closing connection.")
    await websocket.close()

  print("[test_handshake] [client_handshake] Exiting.")

async def server_handshake(server: quicksocket.server.Server):
  print("[test_handshake] [server_handshake] Launching server_handshake.")

  for attempt_num in range(0, 120):
    cli_msgs = server.drain_client_messages()

    # Process messages, if any.
    for cli_msg in cli_msgs:
      print("[test_handshake] [server_handshake] Got client message: {}".format(cli_msg))
      assert(cli_msg == TEMPEST_A4S1_CLIENT_CALL)

      print("[test_handshake] [server_handshake] Sending response: {}".format(TEMPEST_A4S1_SERVER_RESPONSE))
      server.send_messages([TEMPEST_A4S1_SERVER_RESPONSE])
      await asyncio.sleep(0.500)

      print("[test_handshake] [server_handshake] Handshake complete, closing connection.")
      return
  
    await asyncio.sleep(0.050)

  raise Exception("[test_handshake] [server_handshake] Exception: Failed to receive client message in a reasonable amount of time.")

def test_connection():
  port = 59994

  # Start the server and confirm it starts.
  print("Starting.")
  server = quicksocket.server.Server()
  server.start(port)
  time.sleep(0.200)
  assert(server.is_running())

  # Run the asynchronous client and server handshake.
  async def run_tasks():
    cli_hsh_task = asyncio.create_task(client_handshake(port))
    srv_hsh_task = asyncio.create_task(server_handshake(server))
    await cli_hsh_task
    await srv_hsh_task
  asyncio.run(run_tasks())
  print("[test_handshake] [test_connection] Done running tasks.")

  # Stop the server and confirm it stops.
  print("Stopping server.")
  server.stop()
  time.sleep(0.200)
  assert(not server.is_running())

if __name__ == "__main__":
  test_connection()
