#!/usr/bin/env python3
import json
import sys

for line in sys.stdin:
    request = json.loads(line)
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": {"ready": False}}) + "\n")
    sys.stdout.flush()
