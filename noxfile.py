# Noxfile for quicksocket. Runs a basic integration test that confirms quicksocket can start a Websocket server and receive messages from a Python websocket client.

import nox

@nox.session(python=["3.6", "3.7", "3.8", "3.9"])
def test(session):
  session.install("pytest", "websockets")
  session.install(".")
  session.run("pytest")

@nox.session(python=["3.6", "3.7", "3.8", "3.9"])
def lint(session):
  print("TODO: nox lint session")
  pass

@nox.session(python=["3.9"])
def docs(session):
  print("TODO: nox docs session")
  pass
