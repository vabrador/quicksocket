import traceback
from typing import List, Union

from .quicksocket import start_server as BACKEND_start_server
from .quicksocket import is_server_running as BACKEND_is_server_running
from .quicksocket import shutdown_server as BACKEND_shutdown_server
from .quicksocket import drain_new_client_events as BACKEND_drain_new_client_events
from .quicksocket import drain_client_messages as BACKEND_drain_client_messages
from .quicksocket import try_send_messages as BACKEND_try_send_messages

class Server:
  '''Wrapper around the quicksocket module that provides type annotations.'''

  def start(self, port: int):
    BACKEND_start_server(port = port)

  def is_running(self) -> bool:
    running = BACKEND_is_server_running()
    return running

  def stop(self):
    BACKEND_shutdown_server()

  def drain_new_client_events(self) -> List[str]:
    new_client_events: List[str] = BACKEND_drain_new_client_events()
    # for new_client in new_client_events:
    #   print('Drained new client event: {}'.format(new_client))
    
    return new_client_events
  
  def drain_client_messages(self) -> List[Union[str, bytes]]:
    client_msgs: List[Union[str, bytes]] = BACKEND_drain_client_messages()
    return client_msgs

  def send_messages(self, messages: List[Union[str, bytes]]):
    '''If you have more than one message to send, best to send as many of them as you can to the library at once, so any synchronization overhead isn't eaten more than is necessary.'''
    try:
      BACKEND_try_send_messages(messages)
      # print("Successfully sent messages!")
    except BaseException as e:
      print('Exception trying to send messages {}'.format(e))
      print('Traceback: {}'.format(traceback.print_tb(e.__traceback__)))
