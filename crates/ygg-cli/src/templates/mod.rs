pub(crate) const PYTHON_SUBPROCESS_TEMPLATE: &str = r#"#!/usr/bin/env python3
import json
import sys


def respond(payload):
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()


for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "package.handshake":
        respond({"jsonrpc": "2.0", "id": request.get("id"), "result": {"ready": True, "package_protocol_version": "0.1.0"}})
    elif method == "capability.invoke":
        params = request.get("params", {})
        respond({"jsonrpc": "2.0", "id": request.get("id"), "result": {"output": params.get("input")}})
    else:
        respond({"jsonrpc": "2.0", "id": request.get("id"), "error": {"code": "unknown_method", "message": method}})
"#;

pub(crate) fn typescript_subprocess_template(id: &str) -> String {
    format!(
        r#"import {{ serveSubprocessPackage }} from "./package.mjs";

serveSubprocessPackage({{
  onHandshake: () => ({{ ready: true, package_protocol_version: "0.1.0" }}),
  onInvoke: ({{ capability_id, input }}) => {{
    if (capability_id !== "{id}/echo") {{
      throw new Error(`unsupported capability: ${{capability_id}}`);
    }}
    return input ?? null;
  }},
}});
"#
    )
}

pub(crate) fn typescript_package_json(id: &str) -> String {
    format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "type": "module",
  "private": true,
  "scripts": {{
    "check": "tsc --noEmit"
  }},
  "devDependencies": {{}}
}}
"#,
        id.replace('/', "-")
    )
}

pub(crate) const TYPESCRIPT_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "skipLibCheck": true,
    "types": ["node"]
  },
  "include": ["package.ts"]
}
"#;

pub(crate) const TYPESCRIPT_SUBPROCESS_RUNTIME_TEMPLATE: &str = r#"import readline from "node:readline";

function respond(id, payload) {
  process.stdout.write(JSON.stringify({ jsonrpc: "2.0", id, ...payload }) + "\n");
}

export function serveSubprocessPackage(options) {
  const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  rl.on("line", async (line) => {
    let request;
    try {
      request = JSON.parse(line);
    } catch (error) {
      respond(null, { error: { code: "invalid_json", message: String(error) } });
      return;
    }
    try {
      if (request.method === "package.handshake") {
        const result = options.onHandshake
          ? await options.onHandshake(request.params ?? {})
          : { ready: true, package_protocol_version: "0.1.0" };
        respond(request.id, { result });
      } else if (request.method === "capability.invoke") {
        const output = await options.onInvoke(request.params ?? {});
        respond(request.id, { result: { output } });
      } else {
        respond(request.id, { error: { code: "unknown_method", message: request.method ?? "<missing>" } });
      }
    } catch (error) {
      respond(request.id, { error: { code: "package_error", message: String(error) } });
    }
  });
}

serveSubprocessPackage({
  onInvoke: ({ input }) => input ?? null,
});
"#;

/// TypeScript subprocess template for a networked capability package.
/// Demonstrates: network declarations, secret_ref usage, outbound audit.
pub(crate) fn typescript_networked_template(id: &str) -> String {
    format!(
        r#"import {{ serveSubprocessPackage }} from "./package.mjs";
import {{ secretRef, isValidSecretRef, NetworkDeclaration, OutboundAuditHelper }} from "../../sdk/typescript/secure-execution/index.js";

// Example network declaration — package declares which hosts/methods it needs.
const networkDeclarations = [
  new NetworkDeclaration({{
    host: "api.example.com",
    methods: ["GET", "POST"],
    purpose: "model inference",
  }}),
];

// Example outbound audit helper
const auditHelper = new OutboundAuditHelper({{
  packageId: "{id}",
  capabilityId: "{id}/fetch",
}});

serveSubprocessPackage({{
  onHandshake: () => ({{ ready: true, package_protocol_version: "0.1.0" }}),
  onInvoke: ({{ capability_id, input }}) => {{
    if (capability_id === "{id}/fetch") {{
      // Build an audit-safe request payload — no raw secrets
      const payload = auditHelper.buildRequestPayload({{
        destinationHost: "api.example.com",
        method: "POST",
        secretRefsUsed: [secretRef("env", "MY_API_KEY")],
        purpose: "model inference",
      }});
      // Return the plan — no real network call
      return {{
        plan: "would request api.example.com",
        network_declarations: networkDeclarations.map(d => d.toManifestEntry()),
        audit_payload: payload,
        // NOTE: This package does NOT make real network calls.
        // It returns a plan/discovery result only.
      }};
    }}
    if (capability_id === "{id}/echo") {{
      return input ?? null;
    }}
    throw new Error(`unsupported capability: ${{capability_id}}`);
  }},
}});
"#
    )
}

/// TypeScript subprocess template for a streaming capability package.
/// Demonstrates: streaming lifecycle, faux frame sequence, no real inference.
pub(crate) fn typescript_streaming_template(id: &str) -> String {
    format!(
        r#"import {{ serveSubprocessPackage }} from "./package.mjs";
import {{ StreamFrameClient, secretRef }} from "../../sdk/typescript/secure-execution/index.js";

serveSubprocessPackage({{
  onHandshake: () => ({{ ready: true, package_protocol_version: "0.1.0" }}),
  onInvoke: ({{ capability_id, input }}) => {{
    if (capability_id === "{id}/stream-plan") {{
      // Faux streaming lifecycle — no real model inference
      const client = new StreamFrameClient();
      const startFrame = client.start("{id}/stream-plan", {{ prompt_plan: true }});
      const chunk1 = client.chunk({{ token: "faux_1" }});
      const chunk2 = client.chunk({{ token: "faux_2" }});
      const endFrame = client.end();

      return {{
        plan: "streaming capability readiness proof",
        frames: [startFrame, chunk1, chunk2, endFrame],
        secret_ref_example: secretRef("env", "MY_KEY"),
        // NOTE: No real model inference. Frames are faux/demonstration only.
        // This proves the substrate shape (invocation lifecycle, redaction_state,
        // sequence ordering) without coupling to pi runtime or model APIs.
      }};
    }}
    if (capability_id === "{id}/echo") {{
      return input ?? null;
    }}
    throw new Error(`unsupported capability: ${{capability_id}}`);
  }},
}});
"#
    )
}
