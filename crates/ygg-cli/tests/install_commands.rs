use tempfile::tempdir;
use ygg_cli::commands::install::{InstallArgs, OutputFormat};

#[tokio::test(flavor = "multi_thread")]
async fn install_local_fixture() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("yggdrasil");
    ygg_cli::commands::install::run(InstallArgs {
        source: fixture_path(),
        profile: "test".to_string(),
        data_dir: Some(data_dir.clone()),
        require_signed: false,
        strict: false,
        yes: true,
        format: OutputFormat::Human,
        wrap_as_adapter: false,
        workspace_only: false,
    })
    .await
    .unwrap();

    assert!(data_dir.join("profiles/test.lock.toml").exists());
    let lockfile = std::fs::read_to_string(data_dir.join("profiles/test.lock.toml")).unwrap();
    assert!(lockfile.contains("fixture/pkg-local"));
    assert!(data_dir.join("store").is_dir());
}

#[tokio::test(flavor = "multi_thread")]
async fn list_lockfile_and_uninstall_local_fixture() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("yggdrasil");
    ygg_cli::commands::install::run(InstallArgs {
        source: fixture_path(),
        profile: "test".to_string(),
        data_dir: Some(data_dir.clone()),
        require_signed: false,
        strict: false,
        yes: true,
        format: OutputFormat::Human,
        wrap_as_adapter: false,
        workspace_only: false,
    })
    .await
    .unwrap();

    ygg_cli::commands::list_installed::run(ygg_cli::commands::list_installed::ListInstalledArgs {
        profile: "test".to_string(),
        data_dir: Some(data_dir.clone()),
        format: OutputFormat::Json,
    })
    .await
    .unwrap();

    ygg_cli::commands::lockfile::run(ygg_cli::commands::lockfile::LockfileArgs {
        profile: "test".to_string(),
        data_dir: Some(data_dir.clone()),
        check: true,
    })
    .await
    .unwrap();

    ygg_cli::commands::uninstall::run(ygg_cli::commands::uninstall::UninstallArgs {
        package_id: "fixture/pkg-local".to_string(),
        profile: "test".to_string(),
        data_dir: Some(data_dir.clone()),
        keep_data: false,
        delete_data: false,
    })
    .await
    .unwrap();

    let lockfile = std::fs::read_to_string(data_dir.join("profiles/test.lock.toml")).unwrap();
    assert!(!lockfile.contains("fixture/pkg-local"));
}

fn fixture_path() -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/conformance/fixtures/install/pkg-local")
        .to_string_lossy()
        .to_string()
}
