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
