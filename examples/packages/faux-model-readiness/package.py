#!/usr/bin/env python3
"""
Faux Model Readiness Package — No-Network Proof

This package demonstrates the secure execution substrate shape for
model-like capability packages without performing real model inference
or making network calls. It:

- Uses secret_ref for credentials (never raw secrets)
- Declares network permissions with host/method/purpose
- Returns discovery plans (not real API responses)
- Produces faux streaming frames (not real model output)
- Emphasizes public protocol/capability/proposal patterns
"""

import json
import sys
import time


def respond(payload):
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()


for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "package.handshake":
        respond({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "ready": True,
                "package_protocol_version": "0.1.0",
            },
        })
    elif method == "capability.invoke":
        params = request.get("params", {})
        capability_id = params.get("capability_id", "")
        input_data = params.get("input", {})

        if capability_id == "example/faux-model-readiness/discover":
            # Return a discovery plan — no real network call
            # Use secret_ref (not raw secrets) for credential references
            respond({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "output": {
                        "plan_type": "model_discovery",
                        "provider": "hypothetical-model-provider",
                        "status": "plan_only",
                        "network_declaration": {
                            "host": "api.hypothetical-model.example.com",
                            "methods": ["POST"],
                            "purpose": "model inference discovery",
                        },
                        "secret_ref": "secret_ref:env:HYPOTHETICAL_MODEL_API_KEY",
                        "redaction_state": "redacted",
                        "discovery_steps": [
                            "1. Validate profile against provider schema",
                            "2. Check secret_ref resolution for authentication",
                            "3. Plan outbound request to declared host",
                            "4. Return model catalog as discovery result",
                        ],
                        "note": "No real network call made. This is a readiness plan only.",
                    }
                },
            })
        elif capability_id == "example/faux-model-readiness/stream-faux":
            # Return faux streaming frames as a plan — no real model inference
            invocation_id = f"inv_faux_{int(time.time() * 1000)}"
            stream_id = f"str_faux_{int(time.time() * 1000)}"
            respond({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "output": {
                        "plan_type": "streaming_readiness",
                        "invocation_id": invocation_id,
                        "stream_id": stream_id,
                        "faux_frames": [
                            {
                                "frame_type": "start",
                                "sequence": 0,
                                "redaction_state": "not_captured",
                                "payload": {"plan": "faux model stream start"},
                            },
                            {
                                "frame_type": "chunk",
                                "sequence": 1,
                                "redaction_state": "redacted",
                                "payload": {"token": "faux_token_1"},
                            },
                            {
                                "frame_type": "chunk",
                                "sequence": 2,
                                "redaction_state": "redacted",
                                "payload": {"token": "faux_token_2"},
                            },
                            {
                                "frame_type": "end",
                                "sequence": 3,
                                "redaction_state": "not_captured",
                                "payload": None,
                            },
                        ],
                        "secret_ref_example": "secret_ref:env:HYPOTHETICAL_MODEL_API_KEY",
                        "note": "No real model inference. Frames prove substrate shape only.",
                    }
                },
            })
        else:
            respond({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "error": {
                    "code": "unknown_capability",
                    "message": f"unsupported capability: {capability_id}",
                },
            })
    else:
        respond({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "error": {"code": "unknown_method", "message": method},
        })
