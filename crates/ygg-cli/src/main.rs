mod cli;
mod commands;
mod conformance;
mod templates;

use clap::Parser;

use cli::{
    CapabilityCommand, Cli, Command, CompositionCommand, HostCommand, ManifestCommand,
    PackageCommand, PerfCommand,
};
use commands::audit;
use commands::{capability, composition, demo, host, manifest, package, perf};

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
            PackageCommand::InvokeLocal {
                path,
                capability_id,
                input,
            } => package::package_invoke_local(path, capability_id, input).await,
            PackageCommand::Conformance { path } => package::package_conformance(path).await,
            PackageCommand::Reload { path } => package::package_reload(path).await,
            PackageCommand::Install {
                git_url,
                profile,
                package_id,
                reference,
                commit_sha,
                content_hash,
                manifest_path,
            } => {
                package::package_install_git(
                    profile,
                    git_url,
                    package_id,
                    reference,
                    commit_sha,
                    content_hash,
                    manifest_path,
                )
                .await
            }
            PackageCommand::ListInstalled { profile } => {
                package::package_list_installed(profile).await
            }
            PackageCommand::Uninstall {
                package_id,
                profile,
            } => package::package_uninstall_git(profile, package_id).await,
            PackageCommand::Update {
                package_id,
                profile,
                git_url,
                reference,
                commit_sha,
                content_hash,
                manifest_path,
            } => {
                package::package_update_git(
                    profile,
                    package_id,
                    git_url,
                    reference,
                    commit_sha,
                    content_hash,
                    manifest_path,
                )
                .await
            }
            PackageCommand::InspectLockfile { profile } => {
                package::package_inspect_lockfile(profile).await
            }
        },
        Command::Capability { command } => match command {
            CapabilityCommand::Invoke {
                manifest,
                capability_id,
                input,
            } => capability::capability_invoke(manifest, capability_id, input).await,
        },
        Command::Audit(args) => audit::run(args).await,
        Command::InitPackage {
            path,
            id,
            entry,
            language,
            template,
        } => package::init_package(path, id, entry, language, template).await,
        Command::InitComposition { path, id } => composition::init_composition(path, id).await,
        Command::Composition { command } => match command {
            CompositionCommand::Check { path } => composition::composition_check(path).await,
        },
        Command::Conformance {
            list,
            case,
            tag,
            fail_fast,
            slowest,
        } => {
            conformance::run(conformance::ConformanceOptions {
                list,
                case,
                tag,
                fail_fast,
                slowest,
            })
            .await
        }
        Command::PlayCreateDemo => demo::play_create_demo().await,
        Command::PlayableBoardDemo => demo::playable_board_demo().await,
        Command::Perf { command } => match command {
            PerfCommand::Baseline {
                iterations,
                warmup,
                format,
                baseline_out,
                compare,
                threshold_pct,
            } => {
                perf::perf_baseline(perf::BaselineOptions {
                    iterations,
                    warmup,
                    format,
                    baseline_out,
                    compare,
                    threshold_pct,
                })
                .await
            }
        },
    }
}
