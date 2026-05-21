use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
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
        /// Output format: text or json.
        #[arg(long, value_enum, default_value = "text")]
        format: BaselineFormat,
    },
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
    Conformance {
        /// List case ids and tags without executing.
        #[arg(long)]
        list: bool,
        /// Filter cases by substring match on case id (can be repeated).
        #[arg(long, value_name = "PATTERN")]
        case: Vec<String>,
        /// Filter cases by tag (can be repeated; case matches if it has ANY of the specified tags).
        #[arg(long, value_name = "TAG")]
        tag: Vec<String>,
        /// Stop on first failure.
        #[arg(long)]
        fail_fast: bool,
        /// Show the N slowest cases at the end (default 10).
        #[arg(long, default_value = "10")]
        slowest: usize,
    },
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
    Install {
        git_url: String,
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        package_id: String,
        #[arg(long, default_value = "main")]
        reference: String,
        #[arg(long)]
        commit_sha: String,
        #[arg(long)]
        content_hash: String,
        #[arg(long, default_value = "manifest.yaml")]
        manifest_path: String,
    },
    ListInstalled {
        #[arg(long)]
        profile: PathBuf,
    },
    Uninstall {
        package_id: String,
        #[arg(long)]
        profile: PathBuf,
    },
    Update {
        package_id: String,
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        git_url: Option<String>,
        #[arg(long, default_value = "main")]
        reference: String,
        #[arg(long)]
        commit_sha: String,
        #[arg(long)]
        content_hash: String,
        #[arg(long, default_value = "manifest.yaml")]
        manifest_path: String,
    },
    InspectLockfile {
        #[arg(long)]
        profile: PathBuf,
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
    pub(crate) git: HostGitOutboundProfile,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HostGitOutboundProfile {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) executor: HostGitOutboundExecutorKind,
    #[serde(default)]
    pub(crate) allowed_hosts: Vec<String>,
    #[serde(default = "default_true")]
    pub(crate) https_only: bool,
    #[serde(default = "default_git_max_clone_size_mb")]
    pub(crate) max_clone_size_mb: u64,
    #[serde(default = "default_git_timeout_ms")]
    pub(crate) timeout_ms: u64,
    #[serde(default)]
    pub(crate) install_root: Option<PathBuf>,
    #[serde(default)]
    pub(crate) allow_redirects: bool,
}

impl Default for HostGitOutboundProfile {
    fn default() -> Self {
        Self {
            enabled: false,
            executor: HostGitOutboundExecutorKind::DenyAll,
            allowed_hosts: Vec::new(),
            https_only: true,
            max_clone_size_mb: default_git_max_clone_size_mb(),
            timeout_ms: default_git_timeout_ms(),
            install_root: None,
            allow_redirects: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HostGitOutboundExecutorKind {
    DenyAll,
    Fake,
    Real,
}

impl Default for HostGitOutboundExecutorKind {
    fn default() -> Self {
        Self::DenyAll
    }
}

fn default_true() -> bool {
    true
}

fn default_git_max_clone_size_mb() -> u64 {
    64
}

fn default_git_timeout_ms() -> u64 {
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
