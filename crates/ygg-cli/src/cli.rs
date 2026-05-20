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
}

#[derive(Debug, Parser)]
#[command(name = "ygg")]
#[command(about = "Yggdrasil kernel CLI")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
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
    Conformance,
    /// Run the first blank play-creation loop demo.
    PlayCreateDemo,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ManifestCommand {
    Validate { path: PathBuf },
}

#[derive(Debug, Subcommand)]
pub(crate) enum PackageCommand {
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
    /// Local dev reload/restart smoke: load package, restart if supported, print status/logs.
    Reload { path: PathBuf },
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
    pub(crate) autoload: Vec<PathBuf>,
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
