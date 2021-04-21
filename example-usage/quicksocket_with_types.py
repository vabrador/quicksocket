import traceback
from typing import List, Union

import quicksocket

class WebsocketServer:
  '''Wrapper around the quicksocket module that provides type annotations.'''

  def is_running() -> bool:
    running = quicksocket.is_server_running()
    return running

  def start():
    quicksocket.start_server()

  def stop():
    quicksocket.shutdown_server()

  def drain_new_client_events() -> List[str]:
    new_client_events: List[str] = quicksocket.drain_new_client_events()

    for new_client in new_client_events:
      print('Drained new client event: {}'.format(new_client))

    return new_client_events
  
  def drain_client_messages() -> List[Union[str, bytes]]:
    client_msgs: List[Union[str, bytes]] = quicksocket.drain_client_messages()
    return client_msgs

  def send_messages(messages: List[Union[str, bytes]]):
    '''If you have more than one message to send, best to send as many of them as you can to the library at once, so any synchronization overhead isn't eaten more than is necessary.'''
    try:
      quicksocket.try_send_messages(messages)
      # print("Successfully sent messages!")
    except BaseException as e:
      print('Exception trying to send messages {}'.format(e))
      print('Traceback: {}'.format(traceback.print_tb(e.__traceback__)))
