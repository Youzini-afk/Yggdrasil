use tempfile::TempDir;
use tokio::sync::Mutex;
use ygg_cli::cli::HostProfile;
use ygg_cli::commands::host::runtime_config_from_profile;

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

struct DataDirGuard {
    previous: Option<String>,
    _tmp: TempDir,
}

impl DataDirGuard {
    fn new() -> Self {
        let tmp = tempfile::tempdir().expect("create temp data dir");
        let previous = std::env::var("YGG_DATA_DIR").ok();
        std::env::set_var("YGG_DATA_DIR", tmp.path().display().to_string());
        Self {
            previous,
            _tmp: tmp,
        }
    }
}

impl Drop for DataDirGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var("YGG_DATA_DIR", value),
            None => std::env::remove_var("YGG_DATA_DIR"),
        }
    }
}

struct EnvVarGuard(String);

impl EnvVarGuard {
    fn set(name: &str, value: &str) -> Self {
        std::env::set_var(name, value);
        Self(name.to_string())
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        std::env::remove_var(&self.0);
    }
}

fn profile_from_yaml(yaml: &str) -> HostProfile {
    serde_yaml::from_str(yaml).expect("parse host profile")
}

#[tokio::test]
async fn profile_default_installs_composite_resolver() {
    let _lock = ENV_LOCK.lock().await;
    let _data_dir = DataDirGuard::new();
    let profile = profile_from_yaml("title: test\n");

    let config = runtime_config_from_profile(&profile).expect("build runtime config");
    let err = config
        .secret_resolver
        .resolver
        .resolve("secret_ref:store:NONEXISTENT")
        .await
        .expect_err("missing store secret should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("not in store"),
        "default profile should install store resolver, got: {msg}"
    );
    assert!(
        !msg.contains("no secret resolver configured"),
        "default profile must not leave DenyAll resolver installed: {msg}"
    );
}

#[tokio::test]
async fn profile_with_env_allowlist_resolves_listed_var() {
    let _lock = ENV_LOCK.lock().await;
    let _data_dir = DataDirGuard::new();
    let name = format!("YGG_TEST_RESOLVER_PROBE_{}", std::process::id());
    let _env = EnvVarGuard::set(&name, "synthetic-env-secret-value");
    let profile = profile_from_yaml(&format!(
        "title: test\nsecret_resolver:\n  env_allowlist:\n    - {name}\n"
    ));

    let config = runtime_config_from_profile(&profile).expect("build runtime config");
    let resolved = config
        .secret_resolver
        .resolver
        .resolve(&format!("secret_ref:env:{name}"))
        .await
        .expect("listed env var should resolve");
    assert_eq!(resolved, "synthetic-env-secret-value");
}

#[tokio::test]
async fn profile_with_store_disabled_denies_store_refs() {
    let _lock = ENV_LOCK.lock().await;
    let _data_dir = DataDirGuard::new();
    let profile = profile_from_yaml(
        "title: test\nsecret_resolver:\n  store_enabled: false\n  env_allowlist: []\n",
    );

    let config = runtime_config_from_profile(&profile).expect("build runtime config");
    let err = config
        .secret_resolver
        .resolver
        .resolve("secret_ref:store:ANY")
        .await
        .expect_err("store refs should be denied when all resolvers disabled");
    assert!(
        err.to_string().contains("no secret resolver configured"),
        "store-disabled empty profile should fail closed with DenyAll: {err}"
    );
}
