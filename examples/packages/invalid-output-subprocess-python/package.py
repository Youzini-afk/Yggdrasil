#!/usr/bin/env python3
import json
import sys


def respond(payload):
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()


for line in sys.stdin:
    request = json.loads(line)
    if request.get("method") == "package.handshake":
        respond({"jsonrpc": "2.0", "id": request.get("id"), "result": {"ready": True}})
    elif request.get("method") == "capability.invoke":
        respond({"jsonrpc": "2.0", "id": request.get("id"), "result": {"output": {"bad": True}}})
