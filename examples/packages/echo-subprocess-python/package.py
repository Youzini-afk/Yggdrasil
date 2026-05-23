#!/usr/bin/env python3
import json
import sys

bindings = {}


def respond(payload):
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()


for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "package.handshake":
        bindings = request.get("params", {}).get("bindings", {})
        respond({"jsonrpc": "2.0", "id": request.get("id"), "result": {"ready": True, "package_protocol_version": "0.1.0"}})
    elif method == "capability.invoke":
        params = request.get("params", {})
        input_value = params.get("input")
        if isinstance(input_value, dict) and input_value.get("__return_binding"):
            input_value = bindings.get(input_value.get("__return_binding"))
        respond({"jsonrpc": "2.0", "id": request.get("id"), "result": {"output": input_value}})
    else:
        respond({"jsonrpc": "2.0", "id": request.get("id"), "error": {"code": "unknown_method", "message": method}})
