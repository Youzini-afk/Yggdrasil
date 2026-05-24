pub mod cli;
pub mod commands;
pub mod conformance;
pub mod install;
pub mod schema_export;
pub mod templates;

use cli::{
    CapabilityCommand, Cli, Command, CompositionCommand, ConformanceCommand, HostCommand,
    ManifestCommand, PackageCommand, PerfCommand,
};
use commands::audit;
use commands::{
    capability, composition, conformance_package, demo, host, install as install_command,
    list_installed, lockfile, manifest, package, perf, project, uninstall, update,
};

pub async fn run_cli(cli: Cli) -> anyhow::Result<()> {
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
        },
        Command::Capability { command } => match command {
            CapabilityCommand::Invoke {
                manifest,
                capability_id,
                input,
            } => capability::capability_invoke(manifest, capability_id, input).await,
        },
        Command::Audit(args) => audit::run(args).await,
        Command::Install(args) => install_command::run(args).await,
        Command::Uninstall(args) => uninstall::run(args).await,
        Command::Project(args) => project::run(args).await,
        Command::ListInstalled(args) => list_installed::run(args).await,
        Command::Update(args) => update::run(args).await,
        Command::Lockfile(args) => lockfile::run(args).await,
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
        Command::Conformance(args) => match args.command {
            Some(ConformanceCommand::Package(package_args)) => {
                conformance_package::run(package_args).await
            }
            None => {
                conformance::run(conformance::ConformanceOptions {
                    list: args.list,
                    case: args.case,
                    tag: args.tag,
                    fail_fast: args.fail_fast,
                    slowest: args.slowest,
                })
                .await
            }
        },
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
