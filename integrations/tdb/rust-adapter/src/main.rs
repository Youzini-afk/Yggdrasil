use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

const PACKAGE_ID: &str = "official/tdb-rust-adapter";
const PROTOCOL_VERSION: &str = "0.1.0";

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let response = match line {
            Ok(line) => handle_line(&line),
            Err(error) => jsonrpc_error(Value::Null, "read_error", &sanitize(&error.to_string())),
        };
        let _ = writeln!(stdout, "{}", response);
        let _ = stdout.flush();
    }
}

fn handle_line(line: &str) -> Value {
    let request: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(error) => return jsonrpc_error(Value::Null, "parse_error", &sanitize(&error.to_string())),
    };
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request.get("method").and_then(Value::as_str) {
        Some("package.handshake") => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "ready": true,
                "package_protocol_version": PROTOCOL_VERSION,
                "package_id": PACKAGE_ID,
                "real_tdb_available": real_tdb_available(),
            }
        }),
        Some("capability.invoke") => {
            let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
            let capability_id = params.get("capability_id").and_then(Value::as_str).unwrap_or_default();
            let input = params.get("input").cloned().unwrap_or_else(|| json!({}));
            match invoke(capability_id, input) {
                Ok(output) => json!({"jsonrpc": "2.0", "id": id, "result": {"output": output}}),
                Err(error) => jsonrpc_error(id, "capability_error", &sanitize(&error)),
            }
        }
        Some(other) => jsonrpc_error(id, "unknown_method", &sanitize(other)),
        None => jsonrpc_error(id, "missing_method", "request method is required"),
    }
}

fn invoke(capability_id: &str, input: Value) -> Result<Value, String> {
    let local = capability_id.strip_prefix("official/tdb-rust-adapter/").unwrap_or(capability_id);
    match local {
        "describe_real_tdb_adapter" => Ok(describe_adapter()),
        "run_real_tdb_smoke" => run_real_tdb_smoke(input),
        _ => Err(format!("unsupported capability '{local}'")),
    }
}

fn describe_adapter() -> Value {
    json!({
        "kind": "real_tdb_rust_adapter",
        "package_id": PACKAGE_ID,
        "adapter_role": "ordinary_subprocess_retrieval_provider",
        "real_tdb_available": real_tdb_available(),
        "default_build": {
            "triviumdb_dependency_linked": real_tdb_available(),
            "backend_opened": false,
            "filesystem_performed": false,
            "network_performed": false,
            "kernel_storage": false,
            "event_store_backend": false
        },
        "real_feature": "real-tdb",
        "capabilities": [
            "official/tdb-rust-adapter/describe_real_tdb_adapter",
            "official/tdb-rust-adapter/run_real_tdb_smoke"
        ],
        "red_lines": [
            "not_kernel_database",
            "not_event_store",
            "no_raw_backend_path_in_public_output",
            "bounded_vectors_only"
        ]
    })
}

fn run_real_tdb_smoke(_input: Value) -> Result<Value, String> {
    Ok(json!({
        "kind": "real_tdb_smoke_disabled",
        "real_tdb_available": real_tdb_available(),
        "smoke_executed": false,
        "triviumdb_dependency_linked": real_tdb_available(),
        "backend_opened": false,
        "filesystem_performed": false,
        "network_performed": false,
        "reason": "build without real-tdb feature; enable the opt-in local adapter manifest for a real TriviumDB API proof"
    }))
}

fn real_tdb_available() -> bool {
    cfg!(feature = "real-tdb")
}

fn jsonrpc_error(id: Value, code: &str, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn sanitize(message: &str) -> String {
    let mut redacted = message.replace('\n', " ").replace('\r', " ");
    for marker in ["postgres://", "postgresql://", "sk-", "api_key", "password", "secret"] {
        redacted = redacted.replace(marker, "[redacted]");
    }
    redacted.chars().take(240).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_describe_is_safe_stub() {
        let out = describe_adapter();
        assert_eq!(out["kind"], json!("real_tdb_rust_adapter"));
        assert_eq!(out["default_build"]["backend_opened"], json!(false));
    }

    #[test]
    fn default_smoke_does_not_execute() {
        let out = run_real_tdb_smoke(json!({})).unwrap();
        assert_eq!(out["smoke_executed"], json!(false));
        assert_eq!(out["backend_opened"], json!(false));
    }
}
