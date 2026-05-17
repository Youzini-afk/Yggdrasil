mod cli;
mod commands;
mod conformance;
mod templates;

use clap::Parser;

use cli::{
    CapabilityCommand, Cli, Command, CompositionCommand, HostCommand, ManifestCommand,
    PackageCommand,
};
use commands::{
    capability, composition, demo, host, manifest, package,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Demo => demo::demo().await,
        Command::SqliteDemo { path } => demo::sqlite_demo(path).await,
        Command::Serve { bind } => demo::serve(bind).await,
        Command::Host { command } => match command {
            HostCommand::Serve { http, profile } => host::host_serve(http, profile).await,
        },
        Command::HostStdio => host::host_stdio().await,
        Command::Manifest { command } => match command {
            ManifestCommand::Validate { path } => manifest::validate_manifest(path).await,
        },
        Command::Package { command } => match command {
            PackageCommand::Load { path } => package::package_load(path).await,
            PackageCommand::Check { path } => package::package_check(path).await,
            PackageCommand::RunFixture { path } => package::package_run_fixture(path).await,
            PackageCommand::InvokeLocal { path, capability_id, input } => package::package_invoke_local(path, capability_id, input).await,
            PackageCommand::Conformance { path } => package::package_conformance(path).await,
        },
        Command::Capability { command } => match command {
            CapabilityCommand::Invoke { manifest, capability_id, input } => {
                capability::capability_invoke(manifest, capability_id, input).await
            }
        },
        Command::InitPackage { path, id, entry, language } => package::init_package(path, id, entry, language).await,
        Command::InitComposition { path, id } => composition::init_composition(path, id).await,
        Command::Composition { command } => match command {
            CompositionCommand::Check { path } => composition::composition_check(path).await,
        },
        Command::Conformance => conformance::run().await,
        Command::PlayCreateDemo => demo::play_create_demo().await,
    }
}
