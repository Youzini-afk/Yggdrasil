use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Deserialize;

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum PackageTemplate {
    Basic,
    Experience,
    PlayRenderer,
    ForgePanel,
    AssistantAction,
    AssetEditor,
    FullSurface,
    /// Networked capability template with declared network permissions, secret_ref usage, and outbound audit.
    Networked,
    /// Streaming capability template with stream frame lifecycle and faux frame examples.
    Streaming,
    /// Agent runtime template: deterministic/no-network subprocess with streaming run,
    /// proposal draft, trace summary, and echo capabilities; assistant_action + forge_panel surfaces.
    AgentRuntime,
    /// Experience runtime template: deterministic/no-network subprocess with experience
    /// descriptor, state projection, checkpoint, recovery, and Play/Forge/Assist surface
    /// binding capabilities; all four experience surfaces declared.
    ExperienceRuntime,
    /// Playable board template: deterministic/no-network subprocess with launch,
    /// project_state, render_payload, record_player_action, request_change,
    /// create_checkpoint capabilities; all four experience surfaces declared.
    /// Closest to the official/playable-creation-board shape for third-party creators.
    PlayableBoard,
    /// Playable experience template: deterministic/no-network subprocess with launch,
    /// project_state, render_payload, record_player_action, request_change,
    /// create_checkpoint, inspect_checkpoint, draft_recovery capabilities; all four
    /// experience surfaces declared. Slightly richer than playable-board for
    /// experiences that need checkpoint inspection and recovery planning.
    PlayableExperience,
}

#[derive(Debug, Parser)]
#[command(name = "ygg")]
#[command(about = "Yggdrasil kernel CLI")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum BaselineFormat {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub(crate) enum PerfCommand {
    /// Run deterministic performance baseline measurements.
    Baseline {
        /// Number of iterations per scenario (default 10).
        #[arg(long, default_value = "10")]
        iterations: u32,
        /// Number of unrecorded warmup iterations per scenario.
        #[arg(long, default_value = "1")]
        warmup: u32,
        /// Output format: text or json.
        #[arg(long, value_enum, default_value = "text")]
        format: BaselineFormat,
        /// Write JSON baseline output to a file.
        #[arg(long)]
        baseline_out: Option<PathBuf>,
        /// Compare this run against a previous baseline JSON file.
        #[arg(long)]
        compare: Option<PathBuf>,
        /// Regression threshold percentage for compare mode.
        #[arg(long, default_value = "10.0")]
        threshold_pct: f64,
    },
}

#[derive(Debug, Args)]
pub(crate) struct ConformanceArgs {
    /// Run package conformance checks instead of internal kernel conformance.
    #[command(subcommand)]
    pub(crate) command: Option<ConformanceCommand>,
    /// List case ids and tags without executing.
    #[arg(long)]
    pub(crate) list: bool,
    /// Filter cases by substring match on case id (can be repeated).
    #[arg(long, value_name = "PATTERN")]
    pub(crate) case: Vec<String>,
    /// Filter cases by tag (can be repeated; case matches if it has ANY of the specified tags).
    #[arg(long, value_name = "TAG")]
    pub(crate) tag: Vec<String>,
    /// Stop on first failure.
    #[arg(long)]
    pub(crate) fail_fast: bool,
    /// Show the N slowest cases at the end (default 10).
    #[arg(long, default_value = "10")]
    pub(crate) slowest: usize,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConformanceCommand {
    /// Run the public v1 package conformance test kit.
    Package(crate::commands::conformance_package::ConformancePackageArgs),
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
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
    /// Audit declared package authority against observed effects.
    Audit(crate::commands::audit::AuditArgs),
    /// Generate package skeletons.
    InitPackage {
        path: PathBuf,
        #[arg(long, default_value = "example/new-package")]
        id: String,
        #[arg(long, default_value = "rust_inproc")]
        entry: String,
        #[arg(long, default_value = "python")]
        language: String,
        /// Template controlling generated surfaces: basic|experience|play-renderer|forge-panel|assistant-action|asset-editor|full-surface|networked|streaming|agent-runtime.
        /// Defaults to auto-detected from --language (experience if language contains "experience", otherwise basic).
        #[arg(long, value_enum)]
        template: Option<PackageTemplate>,
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
    Conformance(ConformanceArgs),
    /// Run the first blank play-creation loop demo.
    PlayCreateDemo,
    /// Run the playable creation board vertical slice demo (Experience Beta 1).
    PlayableBoardDemo,
    /// Run performance baseline measurements.
    Perf {
        #[command(subcommand)]
        command: PerfCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ManifestCommand {
    Validate { path: PathBuf },
}

#[derive(Debug, Subcommand)]
pub(crate) enum PackageCommand {
    Load {
        path: PathBuf,
    },
    Check {
        path: PathBuf,
    },
    RunFixture {
        path: PathBuf,
    },
    InvokeLocal {
        path: PathBuf,
        capability_id: String,
        #[arg(long, default_value = "{}")]
        input: String,
    },
    Conformance {
        path: PathBuf,
    },
    /// Local dev reload/restart smoke: load package, restart if supported, print status/logs.
    Reload {
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum HostCommand {
    /// Serve a profile-backed host with HTTP /rpc and event SSE.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8787")]
        http: SocketAddr,
        #[arg(long)]
        profile: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum CompositionCommand {
    Check { path: PathBuf },
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct HostProfile {
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) event_store: HostEventStoreProfile,
    #[serde(default)]
    pub(crate) outbound: HostOutboundProfile,
    #[serde(default)]
    pub(crate) autoload: Vec<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct HostOutboundProfile {
    #[serde(default)]
    pub(crate) execute: HostExecuteOutboundProfile,
    #[serde(default)]
    pub(crate) websocket: HostWebSocketOutboundProfile,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HostExecuteOutboundProfile {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) executor: HostExecuteOutboundExecutorKind,
    #[serde(default)]
    pub(crate) allowed_hosts: Vec<String>,
    #[serde(default = "default_true")]
    pub(crate) https_only: bool,
    #[serde(default = "default_execute_timeout_ms")]
    pub(crate) timeout_ms: u64,
    #[serde(default)]
    pub(crate) allow_redirects: bool,
    #[serde(default)]
    pub(crate) allow_insecure_loopback_for_tests: bool,
}

impl Default for HostExecuteOutboundProfile {
    fn default() -> Self {
        Self {
            enabled: false,
            executor: HostExecuteOutboundExecutorKind::DenyAll,
            allowed_hosts: Vec::new(),
            https_only: true,
            timeout_ms: default_execute_timeout_ms(),
            allow_redirects: false,
            allow_insecure_loopback_for_tests: false,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HostExecuteOutboundExecutorKind {
    #[default]
    DenyAll,
    Fake,
    Live,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostWebSocketOutboundProfile {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default = "default_websocket_executor_kind")]
    pub(crate) executor: HostWebSocketOutboundExecutorKind,
    #[serde(default)]
    pub(crate) allowed_hosts: Vec<String>,
    #[serde(default = "default_wss_only")]
    pub(crate) wss_only: bool,
    #[serde(default = "default_max_idle_ms")]
    pub(crate) max_idle_ms: u64,
    #[serde(default = "default_max_duration_ms")]
    pub(crate) max_duration_ms: u64,
    #[serde(default = "default_max_frame_bytes")]
    pub(crate) max_frame_bytes: usize,
    #[serde(default = "default_max_total_bytes_inbound")]
    pub(crate) max_total_bytes_inbound: usize,
    #[serde(default = "default_max_total_bytes_outbound")]
    pub(crate) max_total_bytes_outbound: usize,
    #[serde(default = "default_max_concurrent_connections")]
    pub(crate) max_concurrent_connections: usize,
    #[serde(default)]
    pub(crate) allow_insecure_ws_for_tests: bool,
}

impl Default for HostWebSocketOutboundProfile {
    fn default() -> Self {
        Self {
            enabled: false,
            executor: HostWebSocketOutboundExecutorKind::DenyAll,
            allowed_hosts: Vec::new(),
            wss_only: default_wss_only(),
            max_idle_ms: default_max_idle_ms(),
            max_duration_ms: default_max_duration_ms(),
            max_frame_bytes: default_max_frame_bytes(),
            max_total_bytes_inbound: default_max_total_bytes_inbound(),
            max_total_bytes_outbound: default_max_total_bytes_outbound(),
            max_concurrent_connections: default_max_concurrent_connections(),
            allow_insecure_ws_for_tests: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HostWebSocketOutboundExecutorKind {
    #[default]
    DenyAll,
    Fake,
    Live,
}

fn default_websocket_executor_kind() -> HostWebSocketOutboundExecutorKind {
    HostWebSocketOutboundExecutorKind::DenyAll
}

fn default_wss_only() -> bool {
    true
}

fn default_max_idle_ms() -> u64 {
    60_000
}

fn default_max_duration_ms() -> u64 {
    1_800_000
}

fn default_max_frame_bytes() -> usize {
    65_536
}

fn default_max_total_bytes_inbound() -> usize {
    10_485_760
}

fn default_max_total_bytes_outbound() -> usize {
    10_485_760
}

fn default_max_concurrent_connections() -> usize {
    8
}

fn default_true() -> bool {
    true
}

fn default_execute_timeout_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum HostEventStoreProfile {
    Memory,
    Sqlite { path: PathBuf },
    Postgres { env: String },
}

impl Default for HostEventStoreProfile {
    fn default() -> Self {
        Self::Memory
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CompositionDescriptor {
    pub(crate) id: String,
    pub(crate) version: String,
    pub(crate) entry_surface_id: String,
    #[serde(default)]
    pub(crate) packages: Vec<PathBuf>,
    #[serde(default)]
    pub(crate) required_surfaces: Vec<String>,
    // v2 optional fields (backwards compatible — all defaulted)
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) optional_packages: Vec<PathBuf>,
    #[serde(default)]
    pub(crate) required_capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) default_activation: Option<serde_json::Value>,
    #[serde(default)]
    pub(crate) permission_expectations: Vec<String>,
    #[serde(default)]
    pub(crate) replacement_candidates: Vec<String>,
    #[serde(default)]
    pub(crate) compatibility_notes: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CapabilityCommand {
    Invoke {
        manifest: PathBuf,
        capability_id: String,
        #[arg(long, default_value = "{}")]
        input: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_profile_execute_default_disabled() {
        let profile: HostProfile =
            serde_yaml::from_str("title: test\n").expect("parse empty profile");
        let exec = &profile.outbound.execute;
        assert!(!exec.enabled, "execute.enabled should default to false");
        assert_eq!(
            exec.executor,
            HostExecuteOutboundExecutorKind::DenyAll,
            "execute.executor should default to DenyAll"
        );
    }

    #[test]
    fn host_profile_execute_parses_live_executor() {
        let yaml = r#"
title: test
outbound:
  execute:
    enabled: true
    executor: live
    allowed_hosts:
      - api.openai.com
    https_only: true
    timeout_ms: 30000
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse live executor profile");
        let exec = &profile.outbound.execute;
        assert!(exec.enabled);
        assert_eq!(exec.executor, HostExecuteOutboundExecutorKind::Live);
        assert_eq!(exec.allowed_hosts, vec!["api.openai.com"]);
    }

    #[test]
    fn host_profile_execute_parses_fake_executor() {
        let yaml = r#"
title: test
outbound:
  execute:
    enabled: true
    executor: fake
    allowed_hosts:
      - api.example.com
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse fake executor profile");
        let exec = &profile.outbound.execute;
        assert!(exec.enabled);
        assert_eq!(exec.executor, HostExecuteOutboundExecutorKind::Fake);
    }

    #[test]
    fn host_profile_execute_parses_deny_all_executor() {
        let yaml = r#"
title: test
outbound:
  execute:
    enabled: true
    executor: deny_all
"#;
        let profile: HostProfile =
            serde_yaml::from_str(yaml).expect("parse deny_all executor profile");
        let exec = &profile.outbound.execute;
        assert!(exec.enabled);
        assert_eq!(exec.executor, HostExecuteOutboundExecutorKind::DenyAll);
    }

    #[test]
    fn host_profile_execute_parses_allowed_hosts() {
        let yaml = r#"
title: test
outbound:
  execute:
    enabled: true
    executor: live
    allowed_hosts:
      - api.openai.com
      - api.deepseek.com
      - api.anthropic.com
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse allowed_hosts profile");
        let exec = &profile.outbound.execute;
        assert_eq!(
            exec.allowed_hosts,
            vec!["api.openai.com", "api.deepseek.com", "api.anthropic.com"]
        );
    }

    #[test]
    fn host_profile_execute_https_only_default_true() {
        let yaml = r#"
title: test
outbound:
  execute:
    enabled: true
    executor: live
    allowed_hosts:
      - api.example.com
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse https_only default");
        let exec = &profile.outbound.execute;
        assert!(
            exec.https_only,
            "https_only should default to true when omitted"
        );
    }

    #[test]
    fn host_profile_execute_timeout_ms_default_30000() {
        let yaml = r#"
title: test
outbound:
  execute:
    enabled: true
    executor: live
    allowed_hosts:
      - api.example.com
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse timeout default");
        let exec = &profile.outbound.execute;
        assert_eq!(
            exec.timeout_ms, 30_000,
            "timeout_ms should default to 30000 when omitted"
        );
    }

    #[test]
    fn host_profile_execute_loopback_default_false() {
        let yaml = r#"
title: test
outbound:
  execute:
    enabled: true
    executor: live
    allowed_hosts:
      - api.example.com
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse loopback default");
        let exec = &profile.outbound.execute;
        assert!(
            !exec.allow_insecure_loopback_for_tests,
            "allow_insecure_loopback_for_tests should default to false"
        );
    }

    #[test]
    fn host_profile_execute_defaults_all_via_empty_outbound() {
        // When outbound section is entirely absent, execute should still default correctly
        let profile: HostProfile =
            serde_yaml::from_str("title: test\n").expect("parse profile with no outbound");
        let exec = &profile.outbound.execute;
        assert!(!exec.enabled);
        assert_eq!(exec.executor, HostExecuteOutboundExecutorKind::DenyAll);
        assert!(exec.allowed_hosts.is_empty());
        assert!(exec.https_only);
        assert_eq!(exec.timeout_ms, 30_000);
        assert!(!exec.allow_redirects);
        assert!(!exec.allow_insecure_loopback_for_tests);
    }

    #[test]
    fn host_profile_websocket_default_disabled() {
        let profile: HostProfile = serde_yaml::from_str("title: test\n").expect("parse profile");
        let ws = &profile.outbound.websocket;
        assert!(!ws.enabled);
        assert_eq!(ws.executor, HostWebSocketOutboundExecutorKind::DenyAll);
        assert!(ws.wss_only);
    }

    #[test]
    fn host_profile_websocket_parses_live_executor() {
        let yaml = r#"
title: test
outbound:
  websocket:
    enabled: true
    executor: live
    allowed_hosts:
      - api.openai.com
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse live websocket");
        let ws = &profile.outbound.websocket;
        assert!(ws.enabled);
        assert_eq!(ws.executor, HostWebSocketOutboundExecutorKind::Live);
        assert_eq!(ws.allowed_hosts, vec!["api.openai.com"]);
    }

    #[test]
    fn host_profile_websocket_parses_fake_executor() {
        let yaml = r#"
title: test
outbound:
  websocket:
    enabled: true
    executor: fake
    allowed_hosts:
      - api.example.com
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse fake websocket");
        let ws = &profile.outbound.websocket;
        assert!(ws.enabled);
        assert_eq!(ws.executor, HostWebSocketOutboundExecutorKind::Fake);
    }

    #[test]
    fn host_profile_websocket_parses_deny_all_executor() {
        let yaml = r#"
title: test
outbound:
  websocket:
    enabled: true
    executor: deny_all
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse deny websocket");
        let ws = &profile.outbound.websocket;
        assert!(ws.enabled);
        assert_eq!(ws.executor, HostWebSocketOutboundExecutorKind::DenyAll);
    }

    #[test]
    fn host_profile_websocket_parses_allowed_hosts() {
        let yaml = r#"
title: test
outbound:
  websocket:
    enabled: true
    executor: live
    allowed_hosts:
      - api.openai.com
      - generativelanguage.googleapis.com
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse ws hosts");
        assert_eq!(
            profile.outbound.websocket.allowed_hosts,
            vec!["api.openai.com", "generativelanguage.googleapis.com"]
        );
    }

    #[test]
    fn host_profile_websocket_wss_only_default_true() {
        let profile: HostProfile = serde_yaml::from_str(
            "title: test\noutbound:\n  websocket:\n    enabled: true\n    executor: live\n",
        )
        .expect("parse ws default");
        assert!(profile.outbound.websocket.wss_only);
    }

    #[test]
    fn host_profile_websocket_max_idle_default_60s() {
        let profile: HostProfile = serde_yaml::from_str("title: test\n").expect("parse profile");
        assert_eq!(profile.outbound.websocket.max_idle_ms, 60_000);
    }

    #[test]
    fn host_profile_websocket_max_duration_default_30min() {
        let profile: HostProfile = serde_yaml::from_str("title: test\n").expect("parse profile");
        assert_eq!(profile.outbound.websocket.max_duration_ms, 1_800_000);
    }

    #[test]
    fn host_profile_websocket_max_frame_bytes_default_64kib() {
        let profile: HostProfile = serde_yaml::from_str("title: test\n").expect("parse profile");
        assert_eq!(profile.outbound.websocket.max_frame_bytes, 65_536);
    }

    #[test]
    fn host_profile_websocket_max_concurrent_default_8() {
        let profile: HostProfile = serde_yaml::from_str("title: test\n").expect("parse profile");
        assert_eq!(profile.outbound.websocket.max_concurrent_connections, 8);
    }

    #[test]
    fn host_profile_websocket_round_trips() {
        let yaml = r#"
title: test
outbound:
  websocket:
    enabled: true
    executor: live
    allowed_hosts:
      - api.openai.com
      - generativelanguage.googleapis.com
    wss_only: true
    max_idle_ms: 60000
    max_duration_ms: 1800000
    max_frame_bytes: 65536
    max_total_bytes_inbound: 10485760
    max_total_bytes_outbound: 10485760
    max_concurrent_connections: 8
    allow_insecure_ws_for_tests: false
"#;
        let profile: HostProfile = serde_yaml::from_str(yaml).expect("parse websocket profile");
        let rendered = format!(
            r#"title: test
outbound:
  websocket:
    enabled: {}
    executor: live
    allowed_hosts:
      - {}
      - {}
    wss_only: {}
    max_idle_ms: {}
    max_duration_ms: {}
    max_frame_bytes: {}
    max_total_bytes_inbound: {}
    max_total_bytes_outbound: {}
    max_concurrent_connections: {}
    allow_insecure_ws_for_tests: {}
"#,
            profile.outbound.websocket.enabled,
            profile.outbound.websocket.allowed_hosts[0],
            profile.outbound.websocket.allowed_hosts[1],
            profile.outbound.websocket.wss_only,
            profile.outbound.websocket.max_idle_ms,
            profile.outbound.websocket.max_duration_ms,
            profile.outbound.websocket.max_frame_bytes,
            profile.outbound.websocket.max_total_bytes_inbound,
            profile.outbound.websocket.max_total_bytes_outbound,
            profile.outbound.websocket.max_concurrent_connections,
            profile.outbound.websocket.allow_insecure_ws_for_tests
        );
        let reparsed: HostProfile =
            serde_yaml::from_str(&rendered).expect("reparse websocket profile");
        assert_eq!(
            reparsed.outbound.websocket.executor,
            HostWebSocketOutboundExecutorKind::Live
        );
        assert_eq!(
            reparsed.outbound.websocket.allowed_hosts,
            profile.outbound.websocket.allowed_hosts
        );
        assert_eq!(reparsed.outbound.websocket.max_frame_bytes, 65_536);
    }

    #[test]
    fn forge_alpha_backward_compat() {
        // forge-alpha.yaml must still parse with the new execute field defaulting
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let profile_path = base.join("../../profiles/forge-alpha.yaml");
        let raw = std::fs::read_to_string(&profile_path)
            .unwrap_or_else(|_| panic!("read {:?}", profile_path));
        let profile: HostProfile =
            serde_yaml::from_str(&raw).expect("forge-alpha.yaml should parse");
        let exec = &profile.outbound.execute;
        assert!(
            !exec.enabled,
            "forge-alpha execute should default to disabled"
        );
        assert_eq!(
            exec.executor,
            HostExecuteOutboundExecutorKind::DenyAll,
            "forge-alpha executor should default to DenyAll"
        );
        let ws = &profile.outbound.websocket;
        assert!(
            !ws.enabled,
            "forge-alpha websocket should default to disabled"
        );
        assert_eq!(ws.executor, HostWebSocketOutboundExecutorKind::DenyAll);
    }

    #[test]
    fn forge_with_live_models_example_parses() {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let profile_path = base.join("../../profiles/forge-with-live-models.example.yaml");
        let raw = std::fs::read_to_string(&profile_path)
            .unwrap_or_else(|_| panic!("read {:?}", profile_path));
        let profile: HostProfile =
            serde_yaml::from_str(&raw).expect("forge-with-live-models.example.yaml should parse");
        let exec = &profile.outbound.execute;
        assert!(exec.enabled, "execute should be enabled");
        assert_eq!(exec.executor, HostExecuteOutboundExecutorKind::Live);
        assert!(exec.https_only);
        assert_eq!(exec.timeout_ms, 30_000);
        assert!(!exec.allow_redirects);
        assert!(!exec.allow_insecure_loopback_for_tests);
        assert_eq!(exec.allowed_hosts.len(), 7);
        let ws = &profile.outbound.websocket;
        assert!(
            !ws.enabled,
            "example websocket section should stay disabled by default"
        );
        assert_eq!(ws.executor, HostWebSocketOutboundExecutorKind::DenyAll);
    }

    #[test]
    fn forge_with_live_models_example_uncommented_websocket_parses() {
        let raw = r#"
title: Yggdrasil Forge Host With Live Model Outbound Enabled (Example)
outbound:
  execute:
    enabled: true
    executor: live
    allowed_hosts:
      - api.openai.com
    https_only: true
  websocket:
    enabled: false
    executor: deny_all
    allowed_hosts:
      - api.openai.com
      - generativelanguage.googleapis.com
    wss_only: true
    max_idle_ms: 60000
    max_duration_ms: 1800000
    max_frame_bytes: 65536
    max_total_bytes_inbound: 10485760
    max_total_bytes_outbound: 10485760
    max_concurrent_connections: 8
"#;
        let profile: HostProfile = serde_yaml::from_str(raw).expect("parse uncommented ws example");
        let ws = &profile.outbound.websocket;
        assert!(!ws.enabled);
        assert_eq!(ws.executor, HostWebSocketOutboundExecutorKind::DenyAll);
        assert_eq!(ws.allowed_hosts.len(), 2);
    }
}
