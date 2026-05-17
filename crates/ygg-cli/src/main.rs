use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use ygg_core::{
    CapabilityDescriptor, CapabilityPermissions, EventPermissions, KERNEL_PACKAGE_ID,
    HookSubscription, HookTiming, PackageContributions, PackageEntry, PackageManifest, PermissionSet, SandboxPolicy,
};
use ygg_runtime::{
    AppendEventRequest, CapabilityInvocationRequest, EventStore, InMemoryEventStore,
    OpenSessionRequest, ProtocolContext, ProtocolError, Runtime, RuntimeConfig, SqliteEventStore,
};

#[derive(Debug, Parser)]
#[command(name = "ygg")]
#[command(about = "Yggdrasil kernel CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run a content-free kernel event demo.
    Demo,
    /// Run a durable SQLite-backed kernel event demo.
    SqliteDemo { path: PathBuf },
    /// Run the headless kernel HTTP service.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: SocketAddr,
    },
    /// Run host modes.
    Host {
        #[command(subcommand)]
        command: HostCommand,
    },
    /// Run a JSON-RPC-like kernel protocol loop over stdio.
    HostStdio,
    /// Validate a package manifest file.
    Manifest {
        #[command(subcommand)]
        command: ManifestCommand,
    },
    /// Exercise the in-memory package registry.
    Package {
        #[command(subcommand)]
        command: PackageCommand,
    },
    /// Exercise capability discovery and invocation against a manifest.
    Capability {
        #[command(subcommand)]
        command: CapabilityCommand,
    },
    /// Generate package skeletons.
    InitPackage {
        path: PathBuf,
        #[arg(long, default_value = "example/new-package")]
        id: String,
        #[arg(long, default_value = "rust_inproc")]
        entry: String,
        #[arg(long, default_value = "python")]
        language: String,
    },
    /// Generate a local composition descriptor.
    InitComposition {
        path: PathBuf,
        #[arg(long, default_value = "example/composition")]
        id: String,
    },
    /// Validate composition descriptors.
    Composition {
        #[command(subcommand)]
        command: CompositionCommand,
    },
    /// Run local kernel conformance checks.
    Conformance,
    /// Run the first blank play-creation loop demo.
    PlayCreateDemo,
}

#[derive(Debug, Subcommand)]
enum ManifestCommand {
    Validate { path: PathBuf },
}

#[derive(Debug, Subcommand)]
enum PackageCommand {
    Load { path: PathBuf },
    Check { path: PathBuf },
    RunFixture { path: PathBuf },
    InvokeLocal {
        path: PathBuf,
        capability_id: String,
        #[arg(long, default_value = "{}")]
        input: String,
    },
    Conformance { path: PathBuf },
}

#[derive(Debug, Subcommand)]
enum HostCommand {
    /// Serve a profile-backed host with HTTP /rpc and event SSE.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8787")]
        http: SocketAddr,
        #[arg(long)]
        profile: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum CompositionCommand {
    Check { path: PathBuf },
}

#[derive(Debug, Default, Deserialize)]
struct HostProfile {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    autoload: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct CompositionDescriptor {
    id: String,
    version: String,
    entry_surface_id: String,
    #[serde(default)]
    packages: Vec<PathBuf>,
    #[serde(default)]
    required_surfaces: Vec<String>,
}

#[derive(Debug, Subcommand)]
enum CapabilityCommand {
    Invoke {
        manifest: PathBuf,
        capability_id: String,
        #[arg(long, default_value = "{}")]
        input: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Demo => demo().await,
        Command::SqliteDemo { path } => sqlite_demo(path).await,
        Command::Serve { bind } => serve(bind).await,
        Command::Host { command } => match command {
            HostCommand::Serve { http, profile } => host_serve(http, profile).await,
        },
        Command::HostStdio => host_stdio().await,
        Command::Manifest { command } => match command {
            ManifestCommand::Validate { path } => validate_manifest(path).await,
        },
        Command::Package { command } => match command {
            PackageCommand::Load { path } => package_load(path).await,
            PackageCommand::Check { path } => package_check(path).await,
            PackageCommand::RunFixture { path } => package_run_fixture(path).await,
            PackageCommand::InvokeLocal { path, capability_id, input } => package_invoke_local(path, capability_id, input).await,
            PackageCommand::Conformance { path } => package_conformance(path).await,
        },
        Command::Capability { command } => match command {
            CapabilityCommand::Invoke { manifest, capability_id, input } => {
                capability_invoke(manifest, capability_id, input).await
            }
        },
        Command::InitPackage { path, id, entry, language } => init_package(path, id, entry, language).await,
        Command::InitComposition { path, id } => init_composition(path, id).await,
        Command::Composition { command } => match command {
            CompositionCommand::Check { path } => composition_check(path).await,
        },
        Command::Conformance => conformance().await,
        Command::PlayCreateDemo => play_create_demo().await,
    }
}

async fn read_manifest(path: PathBuf) -> anyhow::Result<PackageManifest> {
    let raw = fs::read_to_string(&path)?;
    let manifest = match path.extension().and_then(|ext| ext.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&raw)?,
        _ => serde_json::from_str(&raw)?,
    };
    Ok(manifest)
}

async fn validate_manifest(path: PathBuf) -> anyhow::Result<()> {
    let manifest = read_manifest(path).await?;
    manifest.validate_basic()?;
    println!("valid manifest: {}@{}", manifest.id, manifest.version);
    Ok(())
}

async fn package_load(path: PathBuf) -> anyhow::Result<()> {
    let manifest = read_manifest(path).await?;
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    let record = runtime.load_package(manifest).await?;
    println!("loaded package: {}@{} ({:?})", record.id, record.version, record.state);
    Ok(())
}

async fn package_check(path: PathBuf) -> anyhow::Result<()> {
    let manifest = read_manifest(path).await?;
    manifest.validate_basic()?;
    println!("package check: {}@{} ok", manifest.id, manifest.version);
    Ok(())
}

async fn package_run_fixture(path: PathBuf) -> anyhow::Result<()> {
    let manifest = read_manifest(path).await?;
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    let record = runtime.load_package(manifest).await?;
    println!("package fixture ready: {} ({:?})", record.id, record.state);
    Ok(())
}

async fn package_invoke_local(path: PathBuf, capability_id: String, input: String) -> anyhow::Result<()> {
    let manifest = read_manifest(path).await?;
    let payload: serde_json::Value = serde_json::from_str(&input)?;
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    runtime.load_package(manifest).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest { capability_id, caller_package_id: None, provider_package_id: None, version: None, input: payload })
        .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn package_conformance(path: PathBuf) -> anyhow::Result<()> {
    let manifest = read_manifest(path).await?;
    manifest.validate_basic()?;
    let capability = manifest
        .provides
        .first()
        .ok_or_else(|| anyhow::anyhow!("package conformance requires at least one provided capability"))?
        .id
        .clone();
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    runtime.load_package(manifest).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: capability,
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"package_conformance": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"package_conformance": true}), "package did not echo conformance payload");
    println!("package conformance: ok");
    Ok(())
}

async fn init_composition(path: PathBuf, id: String) -> anyhow::Result<()> {
    fs::create_dir_all(&path)?;
    fs::write(
        path.join("composition.yaml"),
        format!(
            r#"id: {id}
version: 0.1.0
entry_surface_id: {id}/entry
packages:
  - ../package/manifest.yaml
required_surfaces:
  - experience_entry
  - play_renderer
  - forge_panel
"#
        ),
    )?;
    println!("initialized composition descriptor at {}", path.join("composition.yaml").display());
    Ok(())
}

async fn composition_check(path: PathBuf) -> anyhow::Result<()> {
    let raw = fs::read_to_string(&path)?;
    let composition: CompositionDescriptor = match path.extension().and_then(|ext| ext.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&raw)?,
        _ => serde_json::from_str(&raw)?,
    };
    anyhow::ensure!(!composition.id.trim().is_empty(), "composition id is required");
    anyhow::ensure!(!composition.version.trim().is_empty(), "composition version is required");
    anyhow::ensure!(!composition.entry_surface_id.trim().is_empty(), "composition entry_surface_id is required");
    let base = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut surface_ids = Vec::new();
    let mut slots = Vec::new();
    for package_path in &composition.packages {
        let resolved = if package_path.is_absolute() { package_path.clone() } else { base.join(package_path) };
        let manifest = read_manifest(resolved).await?;
        manifest.validate_basic()?;
        for surface in manifest.contributes.surfaces {
            let slot = serde_json::to_value(&surface.slot)?.as_str().unwrap_or_default().to_string();
            surface_ids.push(surface.id);
            slots.push(slot);
        }
    }
    anyhow::ensure!(surface_ids.iter().any(|id| id == &composition.entry_surface_id), "entry surface not provided by composition packages");
    for required in &composition.required_surfaces {
        anyhow::ensure!(slots.iter().any(|slot| slot == required), "required surface slot '{required}' missing");
    }
    println!("composition check: {}@{} ok", composition.id, composition.version);
    Ok(())
}

async fn capability_invoke(manifest_path: PathBuf, capability_id: String, input: String) -> anyhow::Result<()> {
    let manifest = read_manifest(manifest_path).await?;
    let payload: serde_json::Value = serde_json::from_str(&input)?;
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    runtime.load_package(manifest).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest { capability_id, caller_package_id: None, provider_package_id: None, version: None, input: payload })
        .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn init_package(path: PathBuf, id: String, entry: String, language: String) -> anyhow::Result<()> {
    fs::create_dir_all(&path)?;
    let package_py = path.join("package.py").display().to_string();
    let package_mjs = path.join("package.mjs").display().to_string();
    let is_typescript = language.starts_with("typescript");
    let subprocess_command = if is_typescript {
        format!("    - node\n    - {package_mjs}")
    } else {
        format!("    - python3\n    - {package_py}")
    };
    let surfaces = if language.contains("experience") {
        format!(
            r#"  surfaces:
    - id: {id}/entry
      version: 0.1.0
      slot: experience_entry
      title: Generated Experience
      description: Launchable package entry generated by ygg init-package.
      capability_id: {id}/echo
      activation:
        launch_capability_id: {id}/echo
        session_template:
          labels: [generated, experience]
      approval_policy: user_approval
    - id: {id}/play
      version: 0.1.0
      slot: play_renderer
      title: Generated Play Renderer
      capability_id: {id}/echo
    - id: {id}/forge
      version: 0.1.0
      slot: forge_panel
      title: Generated Forge Panel
      capability_id: {id}/echo
    - id: {id}/assist
      version: 0.1.0
      slot: assistant_action
      title: Generated Assistant Action
      capability_id: {id}/echo
      approval_policy: fork_then_approve
"#
        )
    } else {
        "  surfaces: []\n".to_string()
    };
    let manifest = match entry.as_str() {
        "wasm" => format!(
            r#"schema_version: 1
id: {id}
version: 0.1.0
entry:
  kind: wasm
  module: package.wasm
  abi_version: 1
  memory_limit_mb: 64
provides: []
consumes: []
contributes:
  schemas: []
  hooks: []
  extension_points: []
  surfaces: []
permissions: {{}}
sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 64
  wall_clock_ms: 30000
"#
        ),
        "remote" => format!(
            r#"schema_version: 1
id: {id}
version: 0.1.0
entry:
  kind: remote
  endpoint: https://example.invalid/ygg/package
  auth:
    scheme: none
    config: null
provides: []
consumes: []
contributes:
  schemas: []
  hooks: []
  extension_points: []
  surfaces: []
permissions: {{}}
sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 128
  wall_clock_ms: 30000
"#
        ),
        "subprocess" => format!(
            r#"schema_version: 1
id: {id}
version: 0.1.0
entry:
  kind: subprocess
  command:
{subprocess_command}
  transport: json_rpc_stdio
provides:
  - id: {id}/echo
    version: 0.1.0
    input_schema: {{}}
    output_schema: {{}}
    streaming: false
consumes: []
contributes:
  schemas: []
  hooks: []
  extension_points: []
{surfaces}
permissions: {{}}
sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 128
  wall_clock_ms: 30000
"#
        ),
        _ => format!(
            r#"schema_version: 1
id: {id}
version: 0.1.0
entry:
  kind: rust_inproc
  crate_ref: package-crate
  symbol: register
  abi_version: 1
provides: []
consumes: []
contributes:
  schemas: []
  hooks: []
  extension_points: []
  surfaces: []
permissions: {{}}
sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 128
  wall_clock_ms: 30000
"#
        ),
    };
    fs::write(path.join("manifest.yaml"), manifest)?;
    if entry == "subprocess" && language.starts_with("python") {
        fs::write(path.join("package.py"), PYTHON_SUBPROCESS_TEMPLATE)?;
    } else if entry == "subprocess" && is_typescript {
        fs::write(path.join("package.ts"), typescript_subprocess_template(&id))?;
        fs::write(path.join("package.mjs"), TYPESCRIPT_SUBPROCESS_RUNTIME_TEMPLATE)?;
        fs::write(path.join("tsconfig.json"), TYPESCRIPT_TSCONFIG)?;
        fs::write(path.join("package.json"), typescript_package_json(&id))?;
    }
    fs::write(
        path.join("README.md"),
        format!("# {id}\n\nYggdrasil capability package skeleton.\n\nRun `ygg package conformance manifest.yaml` from this directory.\n"),
    )?;
    println!("initialized package skeleton at {}", path.display());
    Ok(())
}

const PYTHON_SUBPROCESS_TEMPLATE: &str = r#"#!/usr/bin/env python3
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

fn typescript_subprocess_template(id: &str) -> String {
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

fn typescript_package_json(id: &str) -> String {
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

const TYPESCRIPT_TSCONFIG: &str = r#"{
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

const TYPESCRIPT_SUBPROCESS_RUNTIME_TEMPLATE: &str = r#"import readline from "node:readline";

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

async fn conformance() -> anyhow::Result<()> {
    let mut results = Vec::new();
    record_case(&mut results, "session.open_empty", conformance_session_open().await);
    record_case(&mut results, "event.append_authorized", conformance_event_append_authorized().await);
    record_case(
        &mut results,
        "event.append_without_permission_denied",
        conformance_event_append_without_permission_denied().await,
    );
    record_case(
        &mut results,
        "event.kernel_namespace_denied",
        conformance_kernel_namespace_denied().await,
    );
    record_case(
        &mut results,
        "event.read_without_permission_denied",
        conformance_event_read_without_permission_denied().await,
    );
    record_case(
        &mut results,
        "event.closed_session_rejects_append",
        conformance_closed_session_rejects_append().await,
    );
    record_case(&mut results, "event.range_replay", conformance_event_range_replay().await);
    record_case(&mut results, "capability.invoke_rust_inproc", conformance_capability_invoke().await);
    record_case(
        &mut results,
        "capability.ambiguous_provider_denied",
        conformance_ambiguous_provider_denied().await,
    );
    record_case(
        &mut results,
        "capability.explicit_provider_selected",
        conformance_explicit_provider_selected().await,
    );
    record_case(
        &mut results,
        "package.unload_removes_capability",
        conformance_unload_removes_capability().await,
    );
    record_case(&mut results, "official.no_privilege", conformance_official_no_privilege().await);
    record_case(
        &mut results,
        "schema.capability_input_rejects_invalid",
        conformance_capability_schema_rejects_invalid().await,
    );
    record_case(
        &mut results,
        "schema.event_payload_rejects_invalid",
        conformance_event_schema_rejects_invalid().await,
    );
    record_case(
        &mut results,
        "protocol.structured_permission_error",
        conformance_structured_permission_error().await,
    );
    record_case(&mut results, "permission.grant_revoke_audit", conformance_permission_grant_revoke_audit().await);
    record_case(&mut results, "permission.assistant_capability_grant", conformance_assistant_capability_grant().await);
    record_case(
        &mut results,
        "principal.package_cannot_self_assert_writer",
        conformance_principal_cannot_self_assert_writer().await,
    );
    record_case(
        &mut results,
        "principal.package_cannot_self_assert_capability_caller",
        conformance_principal_cannot_self_assert_capability_caller().await,
    );
    record_case(&mut results, "subprocess.load_ready", conformance_subprocess_load_ready().await);
    record_case(&mut results, "subprocess.invoke_echo", conformance_subprocess_invoke_echo().await);
    record_case(&mut results, "package.lifecycle_timeline", conformance_package_lifecycle_timeline().await);
    record_case(&mut results, "package.logs_capture", conformance_package_logs_capture().await);
    record_case(&mut results, "package.restart_subprocess", conformance_package_restart_subprocess().await);
    record_case(&mut results, "host.diagnostics", conformance_host_diagnostics().await);
    record_case(&mut results, "host.profile_autoload", conformance_host_profile_autoload().await);
    record_case(&mut results, "surface.contribution_list", conformance_surface_contribution_list().await);
    record_case(&mut results, "official.foundation_packages", conformance_official_foundation_packages().await);
    record_case(&mut results, "official.assistant_lab_proposal", conformance_official_assistant_lab_proposal().await);
    record_case(&mut results, "play_creation.blank_loop", conformance_blank_play_creation_loop().await);
    record_case(&mut results, "proposal.lifecycle_apply", conformance_proposal_lifecycle_apply().await);
    record_case(&mut results, "proposal.reject_and_apply_denied", conformance_proposal_reject_and_apply_denied().await);
    record_case(&mut results, "asset.put_get_list", conformance_asset_put_get_list().await);
    record_case(&mut results, "session.fork_branch", conformance_session_fork_branch().await);
    record_case(&mut results, "projection.rebuild", conformance_projection_rebuild().await);
    record_case(&mut results, "substrate.sqlite_rehydrate", conformance_sqlite_substrate_rehydrate().await);
    record_case(&mut results, "subprocess.bad_handshake", conformance_subprocess_bad_handshake().await);
    record_case(&mut results, "subprocess.invoke_timeout", conformance_subprocess_timeout().await);
    record_case(
        &mut results,
        "subprocess.invalid_output_schema",
        conformance_subprocess_invalid_output_schema().await,
    );
    record_case(
        &mut results,
        "subprocess.unload_removes_capability",
        conformance_subprocess_unload_removes_capability().await,
    );
    record_case(&mut results, "protocol.call_host_info", conformance_protocol_call_host_info().await);
    record_case(
        &mut results,
        "protocol.call_capability_in_process",
        conformance_protocol_call_capability_in_process().await,
    );
    record_case(&mut results, "hook.ordering_stable", conformance_hook_ordering_stable().await);
    record_case(&mut results, "hook.veto_blocks_event_append", conformance_hook_veto_blocks_event_append().await);
    record_case(&mut results, "hook.metadata_mutation_allowed", conformance_hook_metadata_mutation_allowed().await);
    record_case(&mut results, "hook.package_owned_handler", conformance_hook_package_owned_handler().await);
    record_case(&mut results, "hook.unload_removes_subscription", conformance_hook_unload_removes_subscription().await);
    record_case(
        &mut results,
        "package.generated_subprocess_conformance",
        conformance_generated_subprocess_package().await,
    );
    record_case(
        &mut results,
        "package.generated_typescript_subprocess_conformance",
        conformance_generated_typescript_subprocess_package().await,
    );
    record_case(&mut results, "package.generated_experience_template", conformance_generated_experience_template().await);
    record_case(&mut results, "composition.check_descriptor", conformance_composition_descriptor().await);
    record_case(&mut results, "official.composition_lab", conformance_official_composition_lab().await);
    record_case(&mut results, "official.asset_lab", conformance_official_asset_lab().await);
    record_case(&mut results, "official.projection_lab", conformance_official_projection_lab().await);
    record_case(&mut results, "official.playable_seed", conformance_official_playable_seed().await);
    record_case(&mut results, "official.persona_lab", conformance_official_persona_lab().await);
    record_case(&mut results, "official.knowledge_lab", conformance_official_knowledge_lab().await);
    record_case(&mut results, "official.context_lab", conformance_official_context_lab().await);

    let mut failed = false;
    for (name, result) in &results {
        match result {
            Ok(()) => println!("{name:<45} PASS"),
            Err(err) => {
                failed = true;
                println!("{name:<45} FAIL {err}");
            }
        }
    }
    if failed {
        anyhow::bail!("conformance failed");
    }
    println!("conformance: ok ({} cases)", results.len());
    Ok(())
}

fn record_case(results: &mut Vec<(&'static str, anyhow::Result<()>)>, name: &'static str, result: anyhow::Result<()>) {
    results.push((name, result));
}

fn runtime() -> (Arc<InMemoryEventStore>, Runtime<InMemoryEventStore>) {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    (store, runtime)
}

async fn conformance_session_open() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    let events = store.list_session(&session.id).await?;
    anyhow::ensure!(events.len() == 1, "expected one session-open event");
    Ok(())
}

async fn conformance_event_append_authorized() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/echo", true, true)).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/echo".to_string(),
            kind: "example/echo/conformance.event".to_string(),
            payload: json!({"conformance": true}),
            metadata: json!({}),
        })
        .await?;
    anyhow::ensure!(store.list_session(&session.id).await?.len() == 2, "expected append event");
    Ok(())
}

async fn conformance_event_append_without_permission_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/noappend", true, false)).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/noappend".to_string(),
            kind: "example/noappend/event".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "append without permission unexpectedly succeeded");
    Ok(())
}

async fn conformance_kernel_namespace_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "kernel/forged".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "package wrote kernel namespace");
    Ok(())
}

async fn conformance_event_read_without_permission_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/noread", false, false)).await?;
    let denied = runtime.list_events_for(&session.id, Some(&"example/noread".to_string())).await;
    anyhow::ensure!(denied.is_err(), "event read without permission unexpectedly succeeded");
    Ok(())
}

async fn conformance_closed_session_rejects_append() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    runtime.close_session(session.id.clone()).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "example/writer/event".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "append to closed session unexpectedly succeeded");
    Ok(())
}

async fn conformance_event_range_replay() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/range", true, true)).await?;
    for idx in 0..3 {
        runtime
            .append_event(AppendEventRequest {
                session_id: session.id.clone(),
                writer_package_id: "example/range".to_string(),
                kind: "example/range/event".to_string(),
                payload: json!({"idx": idx}),
                metadata: json!({}),
            })
            .await?;
    }
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.event.list",
            json!({"session_id": session.id, "after_sequence": 1, "limit": 2, "kind_prefix": "example/range"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let events = value.as_array().ok_or_else(|| anyhow::anyhow!("event list did not return array"))?;
    anyhow::ensure!(events.len() == 2, "expected two ranged events, got {}", events.len());
    anyhow::ensure!(events[0]["sequence"] == json!(2), "range did not resume after sequence");
    Ok(())
}

async fn conformance_capability_invoke() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/echo-rust-inproc", "example/echo-rust-inproc/echo")).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/echo-rust-inproc/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"ok": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"ok": true}), "echo output mismatch");
    Ok(())
}

async fn conformance_ambiguous_provider_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/provider-a", "example/shared/echo")).await?;
    runtime.load_package(echo_package("example/provider-b", "example/shared/echo")).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/shared/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "ambiguous route unexpectedly succeeded");
    Ok(())
}

async fn conformance_explicit_provider_selected() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/provider-a", "example/shared/selected")).await?;
    runtime.load_package(echo_package("example/provider-b", "example/shared/selected")).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/shared/selected".to_string(),
            caller_package_id: None,
            provider_package_id: Some("example/provider-b".to_string()),
            version: Some("^0.1".to_string()),
            input: json!({"selected": true}),
        })
        .await?;
    anyhow::ensure!(result.provider_package_id == "example/provider-b", "explicit provider was ignored");
    Ok(())
}

async fn conformance_unload_removes_capability() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/temp", "example/temp/echo")).await?;
    runtime.unload_package(&"example/temp".to_string()).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/temp/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "unloaded capability remained invokable");
    Ok(())
}

async fn conformance_official_no_privilege() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("official/echo", "example/shared/echo")).await?;
    runtime.load_package(echo_package("thirdparty/echo", "example/shared/echo")).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/shared/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "official-looking package won ambiguous route");
    Ok(())
}

async fn conformance_capability_schema_rejects_invalid() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(schema_echo_package(
            "example/schema-echo",
            "example/schema-echo/echo",
            json!({"type": "object", "required": ["ok"]}),
            json!({"type": "object", "required": ["ok"]}),
        ))
        .await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/schema-echo/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "invalid capability input unexpectedly passed");
    Ok(())
}

async fn conformance_event_schema_rejects_invalid() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_schema_package()).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/schema-writer".to_string(),
            kind: "example/schema-writer/event.checked".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "invalid event payload unexpectedly passed");
    Ok(())
}

async fn conformance_structured_permission_error() -> anyhow::Result<()> {
    let error = ProtocolError::from_anyhow(anyhow::anyhow!("package 'example/nope' is not allowed to read events"));
    anyhow::ensure!(error.code == "kernel/error/permission_denied", "wrong error code: {}", error.code);
    Ok(())
}

async fn conformance_permission_grant_revoke_audit() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/grant-reader", true, true)).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/grant-reader".to_string(),
            kind: "example/grant-reader/event".to_string(),
            payload: json!({"ok": true}),
            metadata: json!({}),
        })
        .await?;
    let human = json!({"kind": "human", "user_id": "user/conformance"});
    let human_context = ProtocolContext { principal: serde_json::from_value(human.clone())?, transport: "conformance".to_string() };
    let denied = runtime
        .call_protocol(&human_context, "kernel.event.list", json!({"session_id": session.id}))
        .await;
    anyhow::ensure!(denied.is_err(), "human read should require grant");
    let grant = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.permission.grant",
            json!({"principal": human, "permission": "events.read", "scope": session.id, "reason": "conformance"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let grant_id = grant["id"].as_str().ok_or_else(|| anyhow::anyhow!("grant missing id"))?.to_string();
    let allowed = runtime
        .call_protocol(&human_context, "kernel.event.list", json!({"session_id": session.id}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(allowed.as_array().map(|items| !items.is_empty()).unwrap_or(false), "grant did not allow event read");
    runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.permission.revoke", json!({"grant_id": grant_id}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let audit = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.permission.audit", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(audit.as_array().map(|items| items.len()).unwrap_or(0) >= 2, "permission audit missing grant/revoke events");
    Ok(())
}

async fn conformance_assistant_capability_grant() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/assistant-target", "example/assistant-target/echo")).await?;
    let assistant = json!({"kind": "assistant", "assistant_id": "assistant/conformance", "delegated_user_id": "user/conformance"});
    let assistant_context = ProtocolContext { principal: serde_json::from_value(assistant.clone())?, transport: "conformance".to_string() };
    let denied = runtime
        .call_protocol(
            &assistant_context,
            "kernel.capability.invoke",
            json!({"capability_id": "example/assistant-target/echo", "input": {"ok": true}}),
        )
        .await;
    anyhow::ensure!(denied.is_err(), "assistant invoke should require grant");
    runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.permission.grant",
            json!({"principal": assistant, "permission": "capabilities.invoke", "scope": "example/assistant-target"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let result = runtime
        .call_protocol(
            &assistant_context,
            "kernel.capability.invoke",
            json!({"capability_id": "example/assistant-target/echo", "input": {"ok": true}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(result["output"] == json!({"ok": true}), "assistant grant did not permit invoke");
    Ok(())
}

async fn conformance_principal_cannot_self_assert_writer() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/actual", true, true)).await?;
    let event = runtime
        .append_event_with_context(
            &ProtocolContext::package("example/actual", "conformance"),
            AppendEventRequest {
                session_id: session.id,
                writer_package_id: "example/spoofed".to_string(),
                kind: "example/actual/event".to_string(),
                payload: json!({}),
                metadata: json!({}),
            },
        )
        .await?;
    anyhow::ensure!(event.writer_package_id == "example/actual", "writer spoof was accepted");
    Ok(())
}

async fn conformance_principal_cannot_self_assert_capability_caller() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/echo", "example/echo/echo")).await?;
    runtime.load_package(event_package("example/actual", false, false)).await?;
    let denied = runtime
        .invoke_capability_with_context(
            &ProtocolContext::package("example/actual", "conformance"),
            CapabilityInvocationRequest {
                capability_id: "example/echo/echo".to_string(),
                caller_package_id: None,
                provider_package_id: None,
                version: None,
                input: json!({}),
            },
        )
        .await;
    anyhow::ensure!(denied.is_err(), "caller self-assertion bypassed invoke permission");
    Ok(())
}

async fn conformance_subprocess_load_ready() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let record = runtime.load_package(read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    anyhow::ensure!(record.id == "example/echo-subprocess-python", "wrong package loaded");
    Ok(())
}

async fn conformance_subprocess_invoke_echo() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/echo-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"subprocess": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"subprocess": true}), "subprocess echo mismatch");
    Ok(())
}

async fn conformance_package_lifecycle_timeline() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    let session_id = "kernel_package_example_echo-subprocess-python".to_string();
    let events = store.list_session(&session_id).await?;
    for expected in ["kernel/package.loading", "kernel/package.starting", "kernel/package.ready", "kernel/package.loaded"] {
        anyhow::ensure!(events.iter().any(|event| event.kind == expected), "missing lifecycle event {expected}");
    }
    Ok(())
}

async fn conformance_package_logs_capture() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("examples/packages/logging-subprocess-python/manifest.yaml")).await?).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/logging-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"logs": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"logs": true}), "logging echo mismatch");
    let logs = runtime.package_logs(&"example/logging-subprocess-python".to_string()).await;
    anyhow::ensure!(logs.iter().any(|log| log.line.contains("invoke observed") || log.line.contains("booted")), "stderr logs were not captured");
    Ok(())
}

async fn conformance_package_restart_subprocess() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    let record = runtime.restart_package(&"example/echo-subprocess-python".to_string()).await?;
    anyhow::ensure!(matches!(record.state, ygg_runtime::PackageState::Ready), "restart did not return ready package");
    Ok(())
}

async fn conformance_host_diagnostics() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/diag", "example/diag/echo")).await?;
    let diagnostics = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.host.diagnostics", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(diagnostics["package_count"] == json!(1), "diagnostics package count mismatch");
    Ok(())
}

async fn conformance_host_profile_autoload() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
    load_host_profile(runtime.clone(), PathBuf::from("profiles/forge-alpha.yaml")).await?;
    let packages = runtime.list_packages().await;
    anyhow::ensure!(packages.iter().any(|package| package.id == "example/echo-rust-inproc"), "profile did not autoload rust package");
    anyhow::ensure!(packages.iter().any(|package| package.id == "example/echo-subprocess-python"), "profile did not autoload subprocess package");
    Ok(())
}

async fn conformance_surface_contribution_list() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("examples/packages/echo-rust-inproc/manifest.yaml")).await?).await?;
    runtime.load_package(read_manifest(PathBuf::from("examples/packages/thirdparty-surface-fixture/manifest.yaml")).await?).await?;
    let all = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.surface.contribution.list", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(all.as_array().map(|items| items.len()).unwrap_or(0) >= 5, "surface contributions were not listed");
    let entries = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.surface.contribution.list",
            json!({"slot": "experience_entry"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(entries[0]["package_id"] == json!("thirdparty/surface-fixture"), "third-party entry surface missing");
    anyhow::ensure!(entries[0]["surface"]["activation"]["launch_capability_id"] == json!("thirdparty/surface-fixture/start"), "entry launch capability missing");
    let described = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.surface.contribution.describe",
            json!({"surface_id": "thirdparty/surface-fixture/assist"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(described["surface"]["approval_policy"] == json!("fork_then_approve"), "assistant action policy missing");
    Ok(())
}

async fn conformance_official_foundation_packages() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    for manifest in [
        "packages/official/package-lab/manifest.yaml",
        "packages/official/schema-tools/manifest.yaml",
        "packages/official/event-tools/manifest.yaml",
    ] {
        runtime.load_package(read_manifest(PathBuf::from(manifest)).await?).await?;
    }
    let echo = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/package-lab/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"official": "ordinary"}),
        })
        .await?;
    anyhow::ensure!(echo.output == json!({"official": "ordinary"}), "package-lab echo failed");
    let schema = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/schema-tools/validate".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"schema": {"type": "object"}, "value": {}}),
        })
        .await?;
    anyhow::ensure!(schema.output["valid"] == json!(true), "schema-tools validate failed");
    let events = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/event-tools/summarize".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"events": [{"kind": "x"}, {"kind": "y"}]}),
        })
        .await?;
    anyhow::ensure!(events.output["event_count"] == json!(2), "event-tools summarize failed");
    let surfaces = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.surface.contribution.list", json!({"slot": "forge_panel"}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(surfaces.as_array().map(|items| items.len()).unwrap_or(0) >= 2, "official package surfaces missing");
    Ok(())
}

async fn conformance_official_assistant_lab_proposal() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("packages/official/assistant-lab/manifest.yaml")).await?).await?;
    let assistant = json!({"kind": "assistant", "assistant_id": "assistant/lab", "delegated_user_id": "user/conformance"});
    let assistant_context = ProtocolContext { principal: serde_json::from_value(assistant.clone())?, transport: "conformance".to_string() };
    let denied = runtime
        .call_protocol(
            &assistant_context,
            "kernel.capability.invoke",
            json!({"capability_id": "official/assistant-lab/draft_branch_change", "input": {"change": "try branch"}}),
        )
        .await;
    anyhow::ensure!(denied.is_err(), "assistant package invocation should require grant");
    runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.permission.grant",
            json!({"principal": assistant, "permission": "capabilities.invoke", "scope": "official/assistant-lab"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let proposal = runtime
        .call_protocol(
            &assistant_context,
            "kernel.capability.invoke",
            json!({"capability_id": "official/assistant-lab/draft_branch_change", "input": {"change": "try branch"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(proposal["output"]["requires_user_approval"] == json!(true), "assistant did not return an approval-gated proposal");
    let surfaces = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.surface.contribution.list", json!({"slot": "assistant_action"}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(surfaces.as_array().map(|items| items.len()).unwrap_or(0) == 1, "assistant surface contribution missing");
    Ok(())
}

async fn conformance_blank_play_creation_loop() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    run_blank_play_creation_loop(&runtime).await.map(|_| ())
}

async fn conformance_asset_put_get_list() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let record_value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.asset.put",
            json!({"mime": "application/json", "content": "{\"hello\":true}", "metadata": {"purpose": "conformance"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let asset_id = record_value["id"].as_str().ok_or_else(|| anyhow::anyhow!("asset put returned no id"))?;
    let get_value = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.asset.get", json!({"asset_id": asset_id}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(get_value["content"] == json!("{\"hello\":true}"), "asset get content mismatch");
    let list_value = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.asset.list", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(list_value.as_array().map(|items| items.len()).unwrap_or(0) == 1, "asset list missing record");
    Ok(())
}

async fn conformance_proposal_lifecycle_apply() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime
        .projection_register(ygg_runtime::runtime::ProjectionDefinition {
            id: "proposal/test-projection".to_string(),
            session_id: session.id.clone(),
            source_kind_prefix: Some("kernel/session".to_string()),
            state: json!({}),
        })
        .await?;
    let created = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.proposal.create",
            json!({
                "target_session_id": session.id,
                "required_permissions": ["assets.write", "projections.rebuild"],
                "expected_effects": {"summary": "write asset and rebuild projection"},
                "operations": [
                    {"op": "asset.put", "payload": {"mime": "application/json", "content": "{\"proposal\":true}"}},
                    {"op": "projection.rebuild", "target": "proposal/test-projection"}
                ]
            }),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let proposal_id = created["id"].as_str().ok_or_else(|| anyhow::anyhow!("proposal missing id"))?.to_string();
    let denied = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.apply", json!({"proposal_id": proposal_id}))
        .await;
    anyhow::ensure!(denied.is_err(), "unapproved proposal should not apply");
    runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.approve", json!({"proposal_id": proposal_id, "reason": "conformance"}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let applied = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.apply", json!({"proposal_id": proposal_id}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(applied["status"] == json!("applied"), "proposal did not reach applied status");
    anyhow::ensure!(applied["result"]["operations"].as_array().map(|items| items.len()).unwrap_or(0) == 2, "proposal apply results missing");
    Ok(())
}

async fn conformance_proposal_reject_and_apply_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let created = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.proposal.create",
            json!({"operations": [{"op": "asset.put", "payload": {"content": "{}"}}]}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let proposal_id = created["id"].as_str().ok_or_else(|| anyhow::anyhow!("proposal missing id"))?.to_string();
    runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.reject", json!({"proposal_id": proposal_id, "reason": "conformance"}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let denied = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.apply", json!({"proposal_id": proposal_id}))
        .await;
    anyhow::ensure!(denied.is_err(), "rejected proposal should not apply");
    Ok(())
}

#[derive(Debug)]
struct BlankLoopResult {
    session_id: String,
    branch_id: String,
    asset_id: String,
    projection_id: String,
}

async fn run_blank_play_creation_loop<S: EventStore>(runtime: &Runtime<S>) -> anyhow::Result<BlankLoopResult> {
    for manifest in [
        "packages/official/assistant-lab/manifest.yaml",
        "packages/official/blank-experience/manifest.yaml",
    ] {
        runtime.load_package(read_manifest(PathBuf::from(manifest)).await?).await?;
    }
    let session = runtime
        .open_session(OpenSessionRequest {
            labels: vec!["play-create".to_string()],
            active_package_set: vec!["official/blank-experience".to_string(), "official/assistant-lab".to_string()],
            metadata: json!({"surface": "play"}),
        })
        .await?;
    let seed = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/blank-experience/create_seed".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"title": "Blank Loop", "intent": "prove play-create substrate"}),
        })
        .await?;
    let assistant = json!({"kind": "assistant", "assistant_id": "assistant/blank-loop", "delegated_user_id": "user/demo"});
    runtime
        .call_protocol(
            &ProtocolContext::host_dev("demo"),
            "kernel.permission.grant",
            json!({"principal": assistant, "permission": "capabilities.invoke", "scope": "official/assistant-lab"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let assistant_context = ProtocolContext { principal: serde_json::from_value(assistant)?, transport: "demo".to_string() };
    let proposal = runtime
        .call_protocol(
            &assistant_context,
            "kernel.capability.invoke",
            json!({"capability_id": "official/assistant-lab/draft_branch_change", "input": {"seed": seed.output, "change": "try a first branch"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(proposal["output"]["requires_user_approval"] == json!(true), "assistant proposal must require approval");
    let branch = runtime.fork_session(session.id.clone(), 0, json!({"proposal": proposal["output"].clone()})).await?;
    let asset = runtime
        .put_asset(ygg_runtime::runtime::AssetPutRequest {
            origin_package_id: Some("official/blank-experience".to_string()),
            mime: "application/json".to_string(),
            content: serde_json::to_string(&json!({"seed": seed.output, "branch_id": branch.id}))?,
            metadata: json!({"kind": "blank_experience_seed"}),
        })
        .await?;
    let projection_id = "official/blank-experience/projection/demo".to_string();
    runtime
        .projection_register(ygg_runtime::runtime::ProjectionDefinition {
            id: projection_id.clone(),
            session_id: session.id.clone(),
            source_kind_prefix: Some("kernel/session".to_string()),
            state: json!({}),
        })
        .await?;
    runtime.projection_rebuild(&projection_id).await?;
    Ok(BlankLoopResult { session_id: session.id, branch_id: branch.id, asset_id: asset.id, projection_id })
}

async fn play_create_demo() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    let result = run_blank_play_creation_loop(&runtime).await?;
    println!("blank play-creation loop ok");
    println!("session_id: {}", result.session_id);
    println!("branch_id: {}", result.branch_id);
    println!("asset_id: {}", result.asset_id);
    println!("projection_id: {}", result.projection_id);
    Ok(())
}

async fn conformance_session_fork_branch() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    let branch_value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.session.fork",
            json!({"parent_session_id": session.id, "forked_from_sequence": 0, "metadata": {"why": "try"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(branch_value["parent_session_id"] == json!(session.id), "branch parent mismatch");
    let branches = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.session.branch.list", json!({"session_id": session.id}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(branches.as_array().map(|items| items.len()).unwrap_or(0) == 1, "branch list missing fork");
    Ok(())
}

async fn conformance_projection_rebuild() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/projection", true, true)).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/projection".to_string(),
            kind: "example/projection/event".to_string(),
            payload: json!({"ok": true}),
            metadata: json!({}),
        })
        .await?;
    runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.projection.register",
            json!({"id": "example/projection/state", "session_id": session.id, "source_kind_prefix": "example/projection", "state": {}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let rebuilt = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.projection.rebuild",
            json!({"projection_id": "example/projection/state"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(rebuilt["state"]["event_count"] == json!(1), "projection event count mismatch");
    Ok(())
}

async fn conformance_sqlite_substrate_rehydrate() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-substrate-{}.db", std::process::id()));
    if path.exists() {
        fs::remove_file(&path)?;
    }
    let store = Arc::new(SqliteEventStore::open(&path)?);
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    let asset = runtime
        .put_asset(ygg_runtime::runtime::AssetPutRequest {
            origin_package_id: None,
            mime: "text/plain".to_string(),
            content: "durable".to_string(),
            metadata: json!({"phase": "A"}),
        })
        .await?;
    let branch = runtime.fork_session(session.id.clone(), 0, json!({"durable": true})).await?;
    runtime
        .projection_register(ygg_runtime::runtime::ProjectionDefinition {
            id: "example/durable/projection".to_string(),
            session_id: session.id.clone(),
            source_kind_prefix: Some("kernel/session".to_string()),
            state: json!({}),
        })
        .await?;
    runtime.projection_rebuild("example/durable/projection").await?;
    drop(runtime);
    drop(store);

    let reopened = Arc::new(SqliteEventStore::open(&path)?);
    let hydrated = Runtime::new(reopened, RuntimeConfig::default());
    hydrated.hydrate_substrate_from_events().await?;
    anyhow::ensure!(hydrated.get_asset(&asset.id).await?.content == "durable", "asset did not rehydrate");
    anyhow::ensure!(hydrated.list_branches(&session.id).await.iter().any(|item| item.id == branch.id), "branch did not rehydrate");
    let projection = hydrated.projection_get("example/durable/projection").await?;
    anyhow::ensure!(projection.state["event_count"].as_u64().unwrap_or(0) >= 1, "projection did not rehydrate");
    let _ = fs::remove_file(path);
    Ok(())
}

async fn conformance_subprocess_bad_handshake() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let denied = runtime
        .load_package(read_manifest(PathBuf::from("examples/packages/bad-handshake-subprocess-python/manifest.yaml")).await?)
        .await;
    anyhow::ensure!(denied.is_err(), "bad handshake unexpectedly loaded");
    Ok(())
}

async fn conformance_subprocess_timeout() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("examples/packages/slow-subprocess-python/manifest.yaml")).await?).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/slow-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "slow subprocess did not time out");
    let status = runtime.package_status(&"example/slow-subprocess-python".to_string()).await;
    anyhow::ensure!(matches!(status.map(|record| record.state), Some(ygg_runtime::PackageState::Degraded)), "timeout did not degrade package");
    Ok(())
}

async fn conformance_subprocess_invalid_output_schema() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(read_manifest(PathBuf::from("examples/packages/invalid-output-subprocess-python/manifest.yaml")).await?)
        .await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/invalid-output-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "invalid subprocess output schema passed");
    Ok(())
}

async fn conformance_subprocess_unload_removes_capability() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    runtime.unload_package(&"example/echo-subprocess-python".to_string()).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/echo-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "unloaded subprocess capability remained invokable");
    Ok(())
}

async fn conformance_protocol_call_host_info() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let value = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.host.info", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(value.get("supported_transports").is_some(), "host.info missing transports");
    Ok(())
}

async fn conformance_protocol_call_capability_in_process() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/protocol", "example/protocol/echo")).await?;
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.capability.invoke",
            json!({"capability_id": "example/protocol/echo", "input": {"via": "protocol"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(value.get("output") == Some(&json!({"via": "protocol"})), "protocol invoke mismatch");
    Ok(())
}

async fn conformance_hook_ordering_stable() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(hook_package("example/hook-b", "kernel/event.before_append", "observe", 0)).await?;
    runtime.load_package(hook_package("example/hook-a", "kernel/event.before_append", "observe", 0)).await?;
    let result = runtime.dispatch_extension("kernel/event.before_append", json!({})).await;
    let invoked: Vec<_> = result.invoked.iter().map(|hook| hook.subscriber_package_id.as_str()).collect();
    anyhow::ensure!(invoked == vec!["example/hook-a", "example/hook-b"], "hook order not stable: {invoked:?}");
    Ok(())
}

async fn conformance_hook_veto_blocks_event_append() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    runtime.load_package(hook_package("example/veto", "kernel/event.before_append", "veto", 0)).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "example/writer/event".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "veto hook did not block append");
    Ok(())
}

async fn conformance_hook_metadata_mutation_allowed() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    runtime
        .load_package(hook_package(
            "example/tracer",
            "kernel/event.before_append",
            "metadata_trace",
            0,
        ))
        .await?;
    let event = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "example/writer/event".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await?;
    anyhow::ensure!(event.metadata["hook_trace"] == "example/tracer", "metadata trace missing");
    Ok(())
}

async fn conformance_hook_package_owned_handler() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    runtime.load_package(hook_handler_package("example/hook-owner", "kernel/event.before_append", "example/hook-owner/trace")).await?;
    let event = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "example/writer/event".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await?;
    anyhow::ensure!(event.metadata.get("hook_trace") == Some(&json!("example/hook-owner")), "package-owned hook handler did not patch metadata");
    Ok(())
}

async fn conformance_hook_unload_removes_subscription() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    runtime.load_package(hook_package("example/veto", "kernel/event.before_append", "veto", 0)).await?;
    runtime.unload_package(&"example/veto".to_string()).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "example/writer/event".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await?;
    Ok(())
}

async fn conformance_generated_subprocess_package() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-package-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    init_package(
        path.clone(),
        "example/generated-subprocess".to_string(),
        "subprocess".to_string(),
        "python".to_string(),
    )
    .await?;
    package_check(path.join("manifest.yaml")).await?;
    package_conformance(path.join("manifest.yaml")).await?;
    fs::remove_dir_all(path)?;
    Ok(())
}

async fn conformance_generated_typescript_subprocess_package() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-ts-package-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    init_package(
        path.clone(),
        "example/generated-typescript-subprocess".to_string(),
        "subprocess".to_string(),
        "typescript".to_string(),
    )
    .await?;
    package_check(path.join("manifest.yaml")).await?;
    package_conformance(path.join("manifest.yaml")).await?;
    fs::remove_dir_all(path)?;
    Ok(())
}

async fn conformance_generated_experience_template() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-experience-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    init_package(
        path.clone(),
        "example/generated-experience".to_string(),
        "subprocess".to_string(),
        "typescript-experience".to_string(),
    )
    .await?;
    package_check(path.join("manifest.yaml")).await?;
    package_conformance(path.join("manifest.yaml")).await?;
    let manifest = read_manifest(path.join("manifest.yaml")).await?;
    anyhow::ensure!(manifest.contributes.surfaces.len() >= 4, "experience template did not generate surface descriptors");
    fs::remove_dir_all(path)?;
    Ok(())
}

async fn conformance_composition_descriptor() -> anyhow::Result<()> {
    let root = std::env::temp_dir().join(format!("ygg-composition-{}", std::process::id()));
    let package_path = root.join("package");
    let composition_path = root.join("composition");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    init_package(
        package_path,
        "example/composed-experience".to_string(),
        "subprocess".to_string(),
        "typescript-experience".to_string(),
    )
    .await?;
    init_composition(composition_path.clone(), "example/composed-experience".to_string()).await?;
    composition_check(composition_path.join("composition.yaml")).await?;
    fs::remove_dir_all(root)?;
    Ok(())
}

async fn conformance_official_composition_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("packages/official/composition-lab/manifest.yaml")).await?).await?;
    let plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/composition-lab/launch_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/composition-lab".to_string()),
            version: None,
            input: json!({
                "id": "example/composed-experience",
                "entry_surface_id": "example/composed-experience/entry",
                "packages": ["example/composed-experience"]
            }),
        })
        .await?;
    anyhow::ensure!(plan.output["kind"] == json!("composition_launch_plan"), "composition lab launch_plan returned wrong kind");
    let graph = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/composition-lab/surface_graph".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/composition-lab".to_string()),
            version: None,
            input: json!({"entry_surface_id": "example/composed-experience/entry", "surfaces": [{"slot": "experience_entry"}]}),
        })
        .await?;
    anyhow::ensure!(graph.output["kind"] == json!("composition_surface_graph"), "composition lab surface_graph returned wrong kind");
    Ok(())
}

async fn conformance_official_asset_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("packages/official/asset-lab/manifest.yaml")).await?).await?;
    let preview = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/asset-lab/preview".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/asset-lab".to_string()),
            version: None,
            input: json!({"asset_id": "asset/demo", "mime": "application/json", "content": "{\"hello\":\"world\"}"}),
        })
        .await?;
    anyhow::ensure!(preview.output["kind"] == json!("asset_preview"), "asset lab preview returned wrong kind");
    let import_plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/asset-lab/import_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/asset-lab".to_string()),
            version: None,
            input: json!({"mime": "application/json", "metadata": {"source": "conformance"}}),
        })
        .await?;
    anyhow::ensure!(import_plan.output["requires_user_approval"] == json!(true), "asset import plan must require approval");
    Ok(())
}

async fn conformance_official_projection_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("packages/official/projection-lab/manifest.yaml")).await?).await?;
    let plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/projection-lab/rebuild_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/projection-lab".to_string()),
            version: None,
            input: json!({"projection_id": "example/projection/state", "source_kind_prefix": "example/projection"}),
        })
        .await?;
    anyhow::ensure!(plan.output["kind"] == json!("projection_rebuild_plan"), "projection lab rebuild_plan returned wrong kind");
    let source = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/projection-lab/explain_source_events".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/projection-lab".to_string()),
            version: None,
            input: json!({"projection_id": "example/projection/state", "events": [{"sequence": 1}], "source_kind_prefix": "example/projection"}),
        })
        .await?;
    anyhow::ensure!(source.output["event_count"] == json!(1), "projection lab source event count mismatch");
    Ok(())
}

async fn conformance_official_playable_seed() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("packages/official/playable-seed/manifest.yaml")).await?).await?;
    let launch = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/playable-seed/launch".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/playable-seed".to_string()),
            version: None,
            input: json!({"title": "Conformance Seed"}),
        })
        .await?;
    anyhow::ensure!(launch.output["kind"] == json!("playable_seed_launch"), "playable seed launch returned wrong kind");
    let render = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/playable-seed/render_payload".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/playable-seed".to_string()),
            version: None,
            input: json!({}),
        })
        .await?;
    anyhow::ensure!(render.output["kind"] == json!("playable_seed_render_payload"), "playable seed render returned wrong kind");
    let proposal = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/playable-seed/propose_change".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/playable-seed".to_string()),
            version: None,
            input: json!({"change": "add one seed block"}),
        })
        .await?;
    anyhow::ensure!(proposal.output["requires_user_approval"] == json!(true), "playable seed change must require approval");
    let surfaces = runtime.list_surface_contributions(Some("experience_entry".to_string())).await;
    let has_entry = surfaces
        .as_array()
        .map(|records| records.iter().any(|record| record["package_id"] == json!("official/playable-seed")))
        .unwrap_or(false);
    anyhow::ensure!(has_entry, "playable seed entry surface missing");
    Ok(())
}

async fn conformance_official_persona_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("packages/official/persona-lab/manifest.yaml")).await?).await?;
    let imported = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/persona-lab/import_profile".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/persona-lab".to_string()),
            version: None,
            input: json!({"source": "conformance", "data": {"spec": "chara_card_v2", "data": {"name": "Mira", "description": "Maps dream cities", "extensions": {"unknown": true}}}}),
        })
        .await?;
    anyhow::ensure!(imported.output["kind"] == json!("persona_profile"), "persona import returned wrong kind");
    anyhow::ensure!(imported.output["core"]["name"] == json!("Mira"), "persona import lost name");
    let fragment = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/persona-lab/render_fragment".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/persona-lab".to_string()),
            version: None,
            input: json!({"profile": imported.output}),
        })
        .await?;
    anyhow::ensure!(fragment.output["kind"] == json!("persona_fragment"), "persona render returned wrong kind");
    anyhow::ensure!(fragment.output.get("provenance").is_some(), "persona render missing provenance");
    Ok(())
}

async fn conformance_official_knowledge_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("packages/official/knowledge-lab/manifest.yaml")).await?).await?;
    let imported = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/knowledge-lab/import_collection".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/knowledge-lab".to_string()),
            version: None,
            input: json!({"format": "worldbook-like", "data": {"name": "Dream City", "entries": {"1": {"key": ["bell"], "content": "Alleys rotate."}}}}),
        })
        .await?;
    anyhow::ensure!(imported.output["kind"] == json!("knowledge_collection"), "knowledge import returned wrong kind");
    let matched = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/knowledge-lab/match_entries".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/knowledge-lab".to_string()),
            version: None,
            input: json!({"query": "the bell rings", "entries": imported.output["entries"]}),
        })
        .await?;
    anyhow::ensure!(matched.output["kind"] == json!("knowledge_match_result"), "knowledge match returned wrong kind");
    anyhow::ensure!(matched.output["matches"].as_array().map(|m| !m.is_empty()).unwrap_or(false), "knowledge match missed keyword");
    let plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/knowledge-lab/injection_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/knowledge-lab".to_string()),
            version: None,
            input: json!({"matches": matched.output["matches"]}),
        })
        .await?;
    anyhow::ensure!(plan.output["kind"] == json!("knowledge_injection_plan"), "knowledge plan returned wrong kind");
    anyhow::ensure!(plan.output["plan_only"] == json!(true), "knowledge injection must be plan-only");
    Ok(())
}

async fn conformance_official_context_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(read_manifest(PathBuf::from("packages/official/context-lab/manifest.yaml")).await?).await?;
    let preview = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/context-lab/assemble_preview".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/context-lab".to_string()),
            version: None,
            input: json!({"budget": 20, "sources": [{"id": "short", "text": "fits"}, {"id": "long", "text": "this source should be omitted by budget"}]}),
        })
        .await?;
    anyhow::ensure!(preview.output["kind"] == json!("context_preview"), "context preview returned wrong kind");
    anyhow::ensure!(preview.output["omitted"].as_array().map(|o| !o.is_empty()).unwrap_or(false), "context preview should report omitted sources");
    let rendered = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/context-lab/render_template".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/context-lab".to_string()),
            version: None,
            input: json!({"template": "Hello {{name}}", "variables": {"name": "Yggdrasil"}}),
        })
        .await?;
    anyhow::ensure!(rendered.output["rendered"] == json!("Hello Yggdrasil"), "context template render failed");
    Ok(())
}

fn event_package(id: &str, read: bool, append: bool) -> PackageManifest {
    PackageManifest {
        id: id.to_string(),
        permissions: PermissionSet { events: EventPermissions { read, append }, ..PermissionSet::default() },
        ..demo_event_writer_manifest()
    }
}

fn hook_package(id: &str, extension_point: &str, handler: &str, precedence: i32) -> PackageManifest {
    PackageManifest {
        id: id.to_string(),
        contributes: PackageContributions {
            hooks: vec![HookSubscription {
                extension_point: extension_point.to_string(),
                handler: handler.to_string(),
                timing: HookTiming::Sync,
                precedence,
            }],
            ..PackageContributions::default()
        },
        ..demo_event_writer_manifest()
    }
}

fn hook_handler_package(id: &str, extension_point: &str, handler: &str) -> PackageManifest {
    PackageManifest {
        schema_version: 1,
        id: id.to_string(),
        version: "0.1.0".to_string(),
        display_name: None,
        description: None,
        author: None,
        license: None,
        entry: PackageEntry::RustInproc {
            crate_ref: "example-hook-inproc".to_string(),
            symbol: "register".to_string(),
            abi_version: 1,
        },
        provides: vec![CapabilityDescriptor {
            id: handler.to_string(),
            version: "0.1.0".to_string(),
            input_schema: serde_json::Value::Null,
            output_schema: serde_json::Value::Null,
            streaming: false,
            side_effects: Vec::new(),
            description: None,
        }],
        consumes: Vec::new(),
        contributes: PackageContributions {
            hooks: vec![HookSubscription {
                extension_point: extension_point.to_string(),
                handler: handler.to_string(),
                timing: HookTiming::Sync,
                precedence: 0,
            }],
            ..PackageContributions::default()
        },
        permissions: PermissionSet::default(),
        sandbox_policy: SandboxPolicy::default(),
    }
}

fn echo_package(id: &str, capability_id: &str) -> PackageManifest {
    schema_echo_package(id, capability_id, serde_json::Value::Null, serde_json::Value::Null)
}

fn schema_echo_package(
    id: &str,
    capability_id: &str,
    input_schema: serde_json::Value,
    output_schema: serde_json::Value,
) -> PackageManifest {
    PackageManifest {
        schema_version: 1,
        id: id.to_string(),
        version: "0.1.0".to_string(),
        display_name: None,
        description: None,
        author: None,
        license: None,
        entry: PackageEntry::RustInproc {
            crate_ref: "example-echo-rust-inproc".to_string(),
            symbol: "register".to_string(),
            abi_version: 1,
        },
        provides: vec![CapabilityDescriptor {
            id: capability_id.to_string(),
            version: "0.1.0".to_string(),
            input_schema,
            output_schema,
            streaming: false,
            side_effects: Vec::new(),
            description: None,
        }],
        consumes: Vec::new(),
        contributes: PackageContributions::default(),
        permissions: PermissionSet {
            capabilities: CapabilityPermissions { invoke: vec!["*".to_string()] },
            ..PermissionSet::default()
        },
        sandbox_policy: SandboxPolicy::default(),
    }
}

fn event_schema_package() -> PackageManifest {
    PackageManifest {
        id: "example/schema-writer".to_string(),
        contributes: PackageContributions {
            schemas: vec![ygg_core::SchemaContribution {
                id: "example/schema-writer/event.checked".to_string(),
                schema: json!({"type": "object", "required": ["ok"]}),
            }],
            hooks: Vec::new(),
            extension_points: Vec::new(),
            surfaces: Vec::new(),
        },
        permissions: PermissionSet {
            events: EventPermissions { read: false, append: true },
            ..PermissionSet::default()
        },
        ..demo_event_writer_manifest()
    }
}

async fn demo() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());

    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(demo_event_writer_manifest()).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/echo".to_string(),
            kind: "example/echo/event.demo".to_string(),
            payload: json!({"message": "content-free kernel event"}),
            metadata: json!({"created_by": "ygg-cli demo"}),
        })
        .await?;

    let events = store.list_session(&session.id).await?;

    println!("session_id: {}", session.id);
    println!("kernel_package_id: {KERNEL_PACKAGE_ID}");
    println!("\nevents:");
    for event in events {
        println!("- #{} {} {}", event.sequence, event.writer_package_id, event.kind);
    }

    Ok(())
}

fn demo_event_writer_manifest() -> PackageManifest {
    PackageManifest {
        schema_version: 1,
        id: "example/echo".to_string(),
        version: "0.1.0".to_string(),
        display_name: Some("Demo Event Writer".to_string()),
        description: None,
        author: None,
        license: None,
        entry: PackageEntry::RustInproc {
            crate_ref: "example-echo".to_string(),
            symbol: "register".to_string(),
            abi_version: 1,
        },
        provides: Vec::new(),
        consumes: Vec::new(),
        contributes: PackageContributions::default(),
        permissions: PermissionSet {
            events: EventPermissions { read: false, append: true },
            ..PermissionSet::default()
        },
        sandbox_policy: SandboxPolicy::default(),
    }
}

async fn sqlite_demo(path: PathBuf) -> anyhow::Result<()> {
    let store = Arc::new(SqliteEventStore::open(&path)?);
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(sqlite_event_writer_manifest()).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/sqlite".to_string(),
            kind: "example/sqlite/event.demo".to_string(),
            payload: json!({"durable": true}),
            metadata: json!({}),
        })
        .await?;
    drop(runtime);
    drop(store);

    let reopened = SqliteEventStore::open(&path)?;
    let events = reopened.list_session(&session.id).await?;
    println!("sqlite_path: {}", path.display());
    println!("session_id: {}", session.id);
    for event in events {
        println!("- #{} {} {}", event.sequence, event.writer_package_id, event.kind);
    }
    Ok(())
}

fn sqlite_event_writer_manifest() -> PackageManifest {
    PackageManifest {
        id: "example/sqlite".to_string(),
        ..demo_event_writer_manifest()
    }
}

async fn serve(bind: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Yggdrasil kernel service listening on http://{bind}");
    axum::serve(listener, ygg_service::app()).await?;
    Ok(())
}

async fn host_serve(http: SocketAddr, profile: Option<PathBuf>) -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
    if let Some(profile_path) = profile {
        load_host_profile(runtime.clone(), profile_path).await?;
    }
    let listener = tokio::net::TcpListener::bind(http).await?;
    println!("Yggdrasil host serving http://{http}");
    println!("  RPC: POST http://{http}/rpc");
    println!("  SSE: GET  http://{http}/kernel/event.subscribe/:session_id");
    let app = ygg_service::app_with_state(ygg_service::AppState { runtime });
    axum::serve(listener, app).await?;
    Ok(())
}

async fn load_host_profile(runtime: Arc<Runtime<InMemoryEventStore>>, profile_path: PathBuf) -> anyhow::Result<()> {
    let raw = fs::read_to_string(&profile_path)?;
    let profile: HostProfile = serde_yaml::from_str(&raw)?;
    if let Some(title) = &profile.title {
        println!("loading host profile: {title}");
    }
    let base = profile_path.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    for manifest_path in profile.autoload {
        let resolved = if manifest_path.is_absolute() { manifest_path } else { base.join(manifest_path) };
        let manifest = read_manifest(resolved).await?;
        let record = runtime.load_package(manifest).await?;
        println!("autoloaded package: {}@{} ({:?})", record.id, record.version, record.state);
    }
    Ok(())
}

async fn host_stdio() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    let context = ProtocolContext::host_dev("host_stdio");
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = tokio::io::stdout();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<ygg_runtime::ProtocolRequest>(&line) {
            Ok(request) => match runtime.call_protocol(&context, &request.method, request.params).await {
                Ok(result) => ygg_runtime::ProtocolResponse { id: request.id, result: Some(result), error: None },
                Err(error) => ygg_runtime::ProtocolResponse { id: request.id, result: None, error: Some(error) },
            },
            Err(error) => ygg_runtime::ProtocolResponse {
                id: "invalid".to_string(),
                result: None,
                error: Some(ygg_runtime::ProtocolError::invalid_request(error.to_string())),
            },
        };
        stdout.write_all(serde_json::to_string(&response)?.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }
    Ok(())
}
