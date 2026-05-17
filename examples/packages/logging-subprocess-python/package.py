#!/usr/bin/env python3
import json
import sys

sys.stderr.write("logging package booted\n")
sys.stderr.flush()

for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "package.handshake":
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": {"ready": True, "package_protocol_version": "0.1.0"}}) + "\n")
        sys.stdout.flush()
    elif method == "capability.invoke":
        params = request.get("params", {})
        sys.stderr.write("invoke observed\n")
        sys.stderr.flush()
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": {"output": params.get("input")}}) + "\n")
        sys.stdout.flush()
    else:
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "error": {"code": "unknown_method", "message": method}}) + "\n")
        sys.stdout.flush()
