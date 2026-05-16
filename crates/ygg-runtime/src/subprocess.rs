use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use ygg_core::{PackageEntry, PackageId, PackageManifest, SubprocessTransport};

#[derive(Default)]
pub struct SubprocessSupervisor {
    handles: RwLock<HashMap<PackageId, Arc<SubprocessHandle>>>,
}

pub struct SubprocessHandle {
    package_id: PackageId,
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<BufReader<ChildStdout>>,
    invoke_timeout: Duration,
}

impl SubprocessSupervisor {
    pub async fn start(&self, manifest: &PackageManifest) -> anyhow::Result<()> {
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
            .stderr(std::process::Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("failed to capture subprocess stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("failed to capture subprocess stdout"))?;
        let handle = Arc::new(SubprocessHandle {
            package_id: manifest.id.clone(),
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(BufReader::new(stdout)),
            invoke_timeout: Duration::from_millis(manifest.sandbox_policy.cpu_quota_ms_per_invoke),
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
        let response = match timeout(handshake_timeout, handle.call(request)).await {
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
}

impl SubprocessHandle {
    async fn call(&self, request: Value) -> anyhow::Result<Value> {
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
}
