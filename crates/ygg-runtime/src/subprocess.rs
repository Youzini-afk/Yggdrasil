//! JSON-RPC-over-stdio subprocess supervisor.
//!
//! In addition to host-initiated `package.handshake` and
//! `capability.invoke` calls, subprocess packages may initiate reverse
//! public kernel calls by writing JSON-RPC requests whose method starts with
//! `kernel.` to stdout.  The supervisor dispatches those requests with the
//! caller principal locked to the subprocess package id and writes responses
//! back to the child's stdin.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::{oneshot, Mutex, RwLock};
use tokio::time::timeout;
use ygg_core::{PackageEntry, PackageId, PackageManifest, SubprocessTransport};

use crate::{EventStore, KernelMethod, ProtocolContext, ProtocolError, Runtime};

#[derive(Default)]
pub struct SubprocessSupervisor {
    handles: RwLock<HashMap<PackageId, Arc<SubprocessHandle>>>,
}

pub struct SubprocessHandle {
    package_id: PackageId,
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<BufReader<ChildStdout>>,
    stderr: Mutex<BufReader<ChildStderr>>,
    invoke_timeout: Duration,
    pending_responses: Mutex<HashMap<String, oneshot::Sender<Value>>>,
    reverse_kernel_requests: Mutex<HashSet<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubprocessLogLine {
    pub package_id: PackageId,
    pub stream: String,
    pub line: String,
}

impl SubprocessSupervisor {
    pub async fn start<S>(&self, manifest: &PackageManifest, runtime: Runtime<S>) -> anyhow::Result<()>
    where
        S: EventStore,
    {
        let PackageEntry::Subprocess { command, transport } = &manifest.entry else {
            return Ok(());
        };
        if transport != &SubprocessTransport::JsonRpcStdio {
            anyhow::bail!("subprocess transport '{transport:?}' is not supported yet");
        }
        let (program, args) = command
            .split_first()
            .ok_or_else(|| anyhow::anyhow!("subprocess command must not be empty"))?;

        let mut child = Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("failed to capture subprocess stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("failed to capture subprocess stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("failed to capture subprocess stderr"))?;
        let handle = Arc::new(SubprocessHandle {
            package_id: manifest.id.clone(),
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(BufReader::new(stdout)),
            stderr: Mutex::new(BufReader::new(stderr)),
            invoke_timeout: Duration::from_millis(manifest.sandbox_policy.cpu_quota_ms_per_invoke),
            pending_responses: Mutex::new(HashMap::new()),
            reverse_kernel_requests: Mutex::new(HashSet::new()),
        });

        let handshake_timeout = Duration::from_millis(manifest.sandbox_policy.wall_clock_ms.min(5_000));
        let request = json!({
            "jsonrpc": "2.0",
            "id": "handshake-1",
            "method": "package.handshake",
            "params": {
                "protocol_version": crate::KERNEL_PROTOCOL_VERSION,
                "package_id": manifest.id,
                "manifest_version": manifest.version,
                "permissions": manifest.permissions,
                "capabilities": manifest.provides.iter().map(|capability| capability.id.clone()).collect::<Vec<_>>(),
            }
        });
        let response = match timeout(handshake_timeout, handle.call_direct(request)).await {
            Ok(result) => result,
            Err(_) => {
                handle.kill().await;
                anyhow::bail!("subprocess '{}' handshake timed out", manifest.id);
            }
        }?;
        if response.get("error").is_some() {
            handle.kill().await;
            anyhow::bail!("subprocess '{}' handshake failed: {response}", manifest.id);
        }
        let ready = response
            .get("result")
            .and_then(|result| result.get("ready"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !ready {
            handle.kill().await;
            anyhow::bail!("subprocess '{}' did not report ready", manifest.id);
        }

        let reverse_handle = handle.clone();
        tokio::spawn(async move {
            reverse_handle.pump_reverse_kernel_requests(runtime).await;
        });

        self.handles.write().await.insert(manifest.id.clone(), handle);
        Ok(())
    }

    pub async fn invoke(&self, package_id: &PackageId, capability_id: &str, input: Value) -> anyhow::Result<Value> {
        let handle = self
            .handles
            .read()
            .await
            .get(package_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("subprocess package '{package_id}' is not ready"))?;
        let request = json!({
            "jsonrpc": "2.0",
            "id": "invoke-1",
            "method": "capability.invoke",
            "params": { "capability_id": capability_id, "input": input }
        });
        let response = match timeout(handle.invoke_timeout, handle.call(request)).await {
            Ok(result) => result?,
            Err(_) => {
                handle.pending_responses.lock().await.remove("invoke-1");
                handle.kill().await;
                self.handles.write().await.remove(package_id);
                anyhow::bail!("subprocess package '{package_id}' invoke timed out");
            }
        };
        if let Some(error) = response.get("error") {
            anyhow::bail!("subprocess package '{package_id}' returned error: {error}");
        }
        response
            .get("result")
            .and_then(|result| result.get("output"))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("subprocess package '{package_id}' returned no output"))
    }

    pub async fn stop(&self, package_id: &PackageId) {
        if let Some(handle) = self.handles.write().await.remove(package_id) {
            handle.kill().await;
        }
    }

    pub async fn restart<S>(&self, manifest: &PackageManifest, runtime: Runtime<S>) -> anyhow::Result<()>
    where
        S: EventStore,
    {
        self.stop(&manifest.id).await;
        self.start(manifest, runtime).await
    }

    pub async fn drain_logs(&self, package_id: &PackageId) -> Vec<SubprocessLogLine> {
        let Some(handle) = self.handles.read().await.get(package_id).cloned() else { return Vec::new() };
        handle.drain_logs().await
    }
}

impl SubprocessHandle {
    async fn call(&self, request: Value) -> anyhow::Result<Value> {
        let id = request
            .get("id")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("subprocess request missing id"))?;
        let id_key = id_to_key(&id);
        let (tx, rx) = oneshot::channel();
        self.pending_responses.lock().await.insert(id_key.clone(), tx);

        if let Err(error) = self.write_json_frame(request).await {
            self.pending_responses.lock().await.remove(&id_key);
            return Err(error);
        }

        rx.await.map_err(|_| anyhow::anyhow!("subprocess package '{}' response channel closed", self.package_id))
    }

    async fn call_direct(&self, request: Value) -> anyhow::Result<Value> {
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        drop(stdin);

        let mut line = String::new();
        let mut stdout = self.stdout.lock().await;
        let read = stdout.read_line(&mut line).await?;
        if read == 0 {
            anyhow::bail!("subprocess package '{}' exited", self.package_id);
        }
        Ok(serde_json::from_str(&line)?)
    }

    async fn kill(&self) {
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    async fn drain_logs(&self) -> Vec<SubprocessLogLine> {
        let mut logs = Vec::new();
        let mut stderr = self.stderr.lock().await;
        loop {
            let mut line = String::new();
            match timeout(Duration::from_millis(1), stderr.read_line(&mut line)).await {
                Ok(Ok(read)) if read > 0 => logs.push(SubprocessLogLine {
                    package_id: self.package_id.clone(),
                    stream: "stderr".to_string(),
                    line: line.trim_end().to_string(),
                }),
                _ => break,
            }
        }
        logs
    }

    async fn pump_reverse_kernel_requests<S>(self: Arc<Self>, runtime: Runtime<S>)
    where
        S: EventStore,
    {
        loop {
            let mut line = String::new();
            let read = {
                let mut stdout = self.stdout.lock().await;
                match stdout.read_line(&mut line).await {
                    Ok(read) => read,
                    Err(_) => 0,
                }
            };
            if read == 0 {
                break;
            }

            let Ok(frame) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if frame.get("method").is_none() {
                if let Some(id) = frame.get("id") {
                    let id_key = id_to_key(id);
                    if let Some(tx) = self.pending_responses.lock().await.remove(&id_key) {
                        let _ = tx.send(frame);
                    }
                }
                continue;
            }
            let Some(method) = frame.get("method").and_then(Value::as_str) else {
                continue;
            };
            if !method.starts_with("kernel.") {
                continue;
            }

            let id = frame.get("id").cloned().unwrap_or(Value::Null);
            let request_id = id_to_key(&id);
            self.reverse_kernel_requests.lock().await.insert(request_id.clone());

            let kernel_method: Result<KernelMethod, _> = method.parse();
            let Ok(kernel_method) = kernel_method else {
                let error = ProtocolError::invalid_request(format!(
                    "protocol method '{}' is not a known kernel method",
                    method
                ));
                let _ = self.write_json_frame(json!({"jsonrpc": "2.0", "id": id, "error": error})).await;
                self.reverse_kernel_requests.lock().await.remove(&request_id);
                continue;
            };

            let stream_events = if kernel_method.streaming() {
                Some(runtime.store().subscribe())
            } else {
                None
            };
            let response = dispatch_reverse_kernel_frame(&runtime, &self.package_id, frame).await;
            let stream_id = if kernel_method.streaming() {
                response
                    .get("result")
                    .and_then(|result| result.get("stream_id"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            } else {
                None
            };

            if self.write_json_frame(response).await.is_err() {
                self.reverse_kernel_requests.lock().await.remove(&request_id);
                break;
            }

            if let (Some(stream_id), Some(stream_events)) = (stream_id, stream_events) {
                let stream_handle = self.clone();
                let runtime_for_stream = runtime.clone();
                tokio::spawn(async move {
                    stream_handle.pipe_reverse_stream(runtime_for_stream, id, request_id, stream_id, stream_events).await;
                });
            } else {
                self.reverse_kernel_requests.lock().await.remove(&request_id);
            }
        }
    }

    async fn pipe_reverse_stream<S>(
        self: Arc<Self>,
        runtime: Runtime<S>,
        id: Value,
        request_id: String,
        stream_id: String,
        mut events: tokio::sync::broadcast::Receiver<ygg_core::EventEnvelope>,
    )
    where
        S: EventStore,
    {
        let Some(_record) = runtime.stream_registry().get_invocation_by_stream_id(&stream_id).await else {
            let _ = self
                .write_json_frame(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "kind": "stream.error",
                    "stream_id": stream_id,
                    "error": "stream not found"
                }))
                .await;
            self.reverse_kernel_requests.lock().await.remove(&request_id);
            return;
        };

        let mut seen_sequences = HashSet::new();
        loop {
            let Ok(event) = events.recv().await else {
                break;
            };
            if event.payload.get("stream_id").and_then(Value::as_str) != Some(stream_id.as_str()) {
                continue;
            }

            let frame = match event.kind.as_str() {
                ygg_core::EVENT_STREAM_CHUNK => {
                    let sequence = event.payload.get("sequence").and_then(Value::as_u64)
                        .or_else(|| event.payload.get("outbound_seq").and_then(Value::as_u64))
                        .unwrap_or(event.sequence);
                    if !seen_sequences.insert(sequence) {
                        continue;
                    }
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "kind": "kernel/stream.chunk",
                        "stream_id": stream_id,
                        "sequence": sequence,
                        "data": event.payload,
                    })
                }
                ygg_core::EVENT_STREAM_ENDED => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "kind": "kernel/stream.ended",
                    "stream_id": stream_id,
                    "summary": event.payload,
                }),
                ygg_core::EVENT_STREAM_ERROR => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "kind": "kernel/stream.error",
                    "stream_id": stream_id,
                    "error": event.payload.get("error").cloned().unwrap_or(event.payload),
                }),
                ygg_core::EVENT_STREAM_CANCELLED => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "kind": "kernel/stream.cancelled",
                    "stream_id": stream_id,
                }),
                ygg_core::EVENT_STREAM_TIMEOUT => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "kind": "kernel/stream.timeout",
                    "stream_id": stream_id,
                }),
                _ => continue,
            };

            let terminal = matches!(
                frame.get("kind").and_then(Value::as_str),
                Some("kernel/stream.ended" | "kernel/stream.error" | "kernel/stream.cancelled" | "kernel/stream.timeout")
            );
            if self.write_json_frame(frame).await.is_err() || terminal {
                break;
            }
        }
        self.reverse_kernel_requests.lock().await.remove(&request_id);
    }

    async fn write_json_frame(&self, frame: Value) -> anyhow::Result<()> {
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(serde_json::to_string(&frame)?.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }
}

pub(crate) fn id_to_key(id: &Value) -> String {
    match id {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

/// Dispatch one reverse `kernel.*` JSON-RPC frame from a subprocess child.
/// The caller principal is always locked to `package_id`; any package_id in
/// params is treated as untrusted request data by downstream dispatch.
pub async fn dispatch_reverse_kernel_frame<S>(
    runtime: &Runtime<S>,
    package_id: &str,
    frame: Value,
) -> Value
where
    S: EventStore,
{
    let id = frame.get("id").cloned().unwrap_or(Value::Null);
    let Some(method) = frame.get("method").and_then(Value::as_str) else {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": ProtocolError::invalid_request("reverse kernel frame missing method"),
        });
    };
    if !method.starts_with("kernel.") {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": ProtocolError::invalid_request("reverse subprocess request method must start with kernel."),
        });
    }
    let context = ProtocolContext::package(package_id.to_string(), "subprocess_stdio");
    match runtime
        .call_subprocess_protocol(&context, method, frame.get("params").cloned().unwrap_or(Value::Null))
        .await
    {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err(error) => json!({"jsonrpc": "2.0", "id": id, "error": error}),
    }
}
