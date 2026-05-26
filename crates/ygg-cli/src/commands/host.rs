use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use ygg_runtime::{
    DenyAllWebSocketExecutor, EventStore, FakeOutboundExecutor, FakeWebSocketExecutor,
    InMemoryEventStore, LiveHttpOutboundExecutor, LiveWebSocketExecutor, LiveWebSocketProfile,
    OutboundExecutePolicyConfig, OutboundExecutorConfig, ProtocolContext, Runtime, RuntimeConfig,
    SqliteEventStore, WebSocketExecutor,
};

use super::manifest::read_manifest;
use crate::cli::{
    HostEventStoreProfile, HostExecuteOutboundExecutorKind, HostExecuteOutboundProfile,
    HostProfile, HostSecretResolverProfile, HostWebSocketOutboundExecutorKind,
    HostWebSocketOutboundProfile,
};

impl LiveWebSocketProfile for HostWebSocketOutboundProfile {
    fn allowed_hosts(&self) -> &[String] {
        &self.allowed_hosts
    }

    fn wss_only(&self) -> bool {
        self.wss_only
    }

    fn max_idle_ms(&self) -> u64 {
        self.max_idle_ms
    }

    fn max_duration_ms(&self) -> u64 {
        self.max_duration_ms
    }

    fn max_frame_bytes(&self) -> usize {
        self.max_frame_bytes
    }

    fn max_total_bytes_inbound(&self) -> usize {
        self.max_total_bytes_inbound
    }

    fn max_total_bytes_outbound(&self) -> usize {
        self.max_total_bytes_outbound
    }

    fn max_concurrent_connections(&self) -> usize {
        self.max_concurrent_connections
    }

    fn allow_insecure_ws_for_tests(&self) -> bool {
        self.allow_insecure_ws_for_tests
    }
}

pub(crate) async fn host_serve(
    http: SocketAddr,
    profile: Option<PathBuf>,
    static_dir: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    access_token: Option<String>,
) -> Result<()> {
    if let Some(data_dir) = data_dir.as_ref() {
        println!("host data dir: {}", data_dir.display());
        std::env::set_var("YGG_DATA_DIR", data_dir);
        ygg_core::paths::ensure_initialized().with_context(|| {
            format!("failed to initialize data directory {}", data_dir.display())
        })?;
    }
    if let Some(profile_path) = profile {
        println!("host profile: {}", profile_path.display());
        let raw = fs::read_to_string(&profile_path)
            .with_context(|| format!("failed to read host profile {}", profile_path.display()))?;
        let profile: HostProfile = serde_yaml::from_str(&raw)
            .with_context(|| format!("failed to parse host profile {}", profile_path.display()))?;
        let mut runtime_config = runtime_config_from_profile(&profile)?;
        register_profile_package_roots(&mut runtime_config, &profile, Some(&profile_path)).await?;
        match &profile.event_store {
            HostEventStoreProfile::Memory => {
                let runtime = Arc::new(Runtime::new(
                    Arc::new(InMemoryEventStore::default()),
                    runtime_config,
                ));
                load_profile_packages(runtime.clone(), profile, profile_path.clone()).await?;
                serve_runtime(
                    http,
                    runtime,
                    "memory",
                    Some(&profile_path),
                    static_dir,
                    access_token,
                )
                .await
            }
            HostEventStoreProfile::Sqlite { path } => {
                let resolved = resolve_profile_path(&profile_path, path.clone());
                if let Some(parent) = resolved.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!(
                            "failed to create event store directory {}",
                            parent.display()
                        )
                    })?;
                }
                let runtime = Arc::new(Runtime::new(
                    Arc::new(SqliteEventStore::open(&resolved).with_context(|| {
                        format!("failed to open sqlite event store {}", resolved.display())
                    })?),
                    runtime_config,
                ));
                load_profile_packages(runtime.clone(), profile, profile_path.clone()).await?;
                serve_runtime(
                    http,
                    runtime,
                    "sqlite",
                    Some(&profile_path),
                    static_dir,
                    access_token,
                )
                .await
            }
            HostEventStoreProfile::Postgres { env } => {
                #[cfg(feature = "postgres")]
                {
                    let url = std::env::var(env).map_err(|_| {
                        anyhow::anyhow!(
                            "postgres event store env ref unavailable (details redacted)"
                        )
                    })?;
                    let store = ygg_runtime::PostgresEventStore::connect(&url).await?;
                    let runtime = Arc::new(Runtime::new(Arc::new(store), runtime_config));
                    load_profile_packages(runtime.clone(), profile, profile_path).await?;
                    serve_runtime(
                        http,
                        runtime,
                        "postgres",
                        Some(&profile_path),
                        static_dir,
                        access_token,
                    )
                    .await
                }
                #[cfg(not(feature = "postgres"))]
                {
                    let _ = env;
                    anyhow::bail!("postgres event store requested but this binary was built without postgres support")
                }
            }
        }
    } else {
        let runtime = Arc::new(Runtime::new(
            Arc::new(InMemoryEventStore::default()),
            RuntimeConfig::default(),
        ));
        serve_runtime(http, runtime, "memory", None, static_dir, access_token).await
    }
}

pub fn runtime_config_from_profile(profile: &HostProfile) -> Result<RuntimeConfig> {
    validate_execute_outbound_profile(&profile.outbound.execute)?;
    validate_websocket_outbound_profile(&profile.outbound.websocket)?;

    let mut config = RuntimeConfig::default();
    // Y1: Wire outbound.execute profile into RuntimeConfig
    let exec = &profile.outbound.execute;
    config.outbound_execute_policy = OutboundExecutePolicyConfig {
        enabled: exec.enabled,
        allowed_hosts: exec.allowed_hosts.clone(),
        https_only: exec.https_only,
        timeout_ms: exec.timeout_ms,
        allow_redirects: exec.allow_redirects,
        allow_insecure_loopback_for_tests: exec.allow_insecure_loopback_for_tests,
    };
    config.outbound_executor = build_outbound_execute_executor(exec)?;

    config.outbound_websocket_executor =
        build_outbound_websocket_executor(&profile.outbound.websocket)?;

    config.secret_resolver = build_secret_resolver(&profile.secret_resolver)?;
    config.surface_dev_paths = profile.surface_dev_paths.clone();

    Ok(config)
}

async fn register_profile_package_roots(
    config: &mut RuntimeConfig,
    profile: &HostProfile,
    profile_path: Option<&Path>,
) -> Result<()> {
    let base = profile_path
        .and_then(Path::parent)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    for manifest_path in &profile.autoload {
        let resolved = if manifest_path.is_absolute() {
            manifest_path.clone()
        } else {
            base.join(manifest_path)
        };
        let manifest = read_manifest(resolved.clone()).await.with_context(|| {
            format!(
                "failed to register package root from manifest {}",
                resolved.display()
            )
        })?;
        if let Some(parent) = resolved.parent() {
            let root = std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
            config.package_roots.insert(manifest.id.clone(), root);
        }
    }
    Ok(())
}

pub(crate) fn build_secret_resolver(
    profile: &HostSecretResolverProfile,
) -> Result<ygg_runtime::SecretResolverConfig> {
    use std::collections::HashSet;
    use ygg_runtime::{
        CompositeSecretResolver, DenyAllSecretResolver, EnvSecretResolver, SecretResolverConfig,
        StoreSecretResolver,
    };

    // If both env and store are off, fall back to DenyAll for safety.
    if profile.env_allowlist.is_empty() && !profile.store_enabled {
        return Ok(SecretResolverConfig::with_resolver(Arc::new(
            DenyAllSecretResolver,
        )));
    }

    let mut composite = CompositeSecretResolver::new();

    if !profile.env_allowlist.is_empty() {
        let allowlist: HashSet<String> = profile.env_allowlist.iter().cloned().collect();
        composite = composite.with_env(Arc::new(EnvSecretResolver::new(allowlist)));
    }

    let platform_store = if profile.store_enabled {
        // StoreSecretResolver::new() resolves the default store path via
        // ygg_core::paths::secret_store_path(). It can fail if data_dir
        // resolution fails on this system — propagate that error.
        let r = Arc::new(StoreSecretResolver::new()?);
        composite = composite.with_store(r.clone());
        Some(r)
    } else {
        None
    };

    // Always wire the project resolver when a composite is active. Platform
    // fallback is whichever StoreSecretResolver we just built (or none).
    let project_resolver = ygg_runtime::ProjectStoreSecretResolver::new(|| {
        ygg_runtime::ACTIVE_PROJECT_SCOPE
            .try_with(|scope| scope.clone())
            .ok()
    });
    let project_resolver = if let Some(platform) = platform_store {
        project_resolver.with_platform_fallback(platform)
    } else {
        project_resolver
    };
    composite = composite.with_project(Arc::new(project_resolver));

    Ok(SecretResolverConfig::with_resolver(Arc::new(composite)))
}

/// Build the `OutboundExecutorConfig` from the execute profile section (Y1).
///
/// - If `enabled` is false, always returns `DenyAll` (fail-closed).
/// - If `enabled` is true, selects based on `executor` field:
///   - `deny_all` → DenyAll
///   - `fake` → Custom(FakeOutboundExecutor)
///   - `live` → LiveHttp(config built from profile fields)
pub(crate) fn build_outbound_execute_executor(
    config: &HostExecuteOutboundProfile,
) -> Result<OutboundExecutorConfig> {
    if !config.enabled {
        return Ok(OutboundExecutorConfig::DenyAll);
    }
    match config.executor {
        HostExecuteOutboundExecutorKind::DenyAll => Ok(OutboundExecutorConfig::DenyAll),
        HostExecuteOutboundExecutorKind::Fake => Ok(OutboundExecutorConfig::Custom(Arc::new(
            FakeOutboundExecutor::new(),
        ))),
        HostExecuteOutboundExecutorKind::Live => {
            let executor = LiveHttpOutboundExecutor::new_from_profile(
                config.https_only,
                config.timeout_ms,
                config.allow_redirects,
                config.allow_insecure_loopback_for_tests,
            )?;
            Ok(OutboundExecutorConfig::Custom(Arc::new(executor)))
        }
    }
}

pub(crate) fn build_outbound_websocket_executor(
    profile: &HostWebSocketOutboundProfile,
) -> Result<Arc<dyn WebSocketExecutor>> {
    if !profile.enabled {
        return Ok(Arc::new(DenyAllWebSocketExecutor));
    }
    match profile.executor {
        HostWebSocketOutboundExecutorKind::DenyAll => Ok(Arc::new(DenyAllWebSocketExecutor)),
        HostWebSocketOutboundExecutorKind::Fake => Ok(Arc::new(FakeWebSocketExecutor::new())),
        HostWebSocketOutboundExecutorKind::Live => {
            let executor = LiveWebSocketExecutor::new_from_profile(profile)?;
            Ok(Arc::new(executor))
        }
    }
}

/// Validate the execute outbound profile section (Y1).
///
/// Enforces fail-closed constraints:
/// - `timeout_ms` must be > 0 when enabled
/// - `allowed_hosts` must not be empty when enabled with a non-deny_all executor
/// - `allowed_hosts` must not contain empty or wildcard hosts
/// - `https_only=false` is not supported (HTTPS-only is the only safe default)
/// - `allow_redirects=true` is not supported (redirects fail closed)
pub(crate) fn validate_execute_outbound_profile(exec: &HostExecuteOutboundProfile) -> Result<()> {
    if !exec.https_only {
        anyhow::bail!(
            "outbound.execute.https_only=false is not supported; live outbound is HTTPS-only"
        )
    }
    if exec.allow_redirects {
        anyhow::bail!(
            "outbound.execute.allow_redirects=true is not supported; redirects fail closed"
        )
    }
    if exec.timeout_ms == 0 {
        anyhow::bail!("outbound.execute.timeout_ms must be greater than zero")
    }
    if !exec.enabled {
        return Ok(());
    }
    if !matches!(exec.executor, HostExecuteOutboundExecutorKind::DenyAll)
        && exec.allowed_hosts.is_empty()
    {
        anyhow::bail!(
            "outbound.execute.allowed_hosts is required when execute outbound is enabled with a non-deny_all executor"
        )
    }
    if exec
        .allowed_hosts
        .iter()
        .any(|host| host.trim().is_empty() || host == "*")
    {
        anyhow::bail!(
            "outbound.execute.allowed_hosts must not contain empty hosts or wildcard hosts"
        )
    }
    Ok(())
}

pub(crate) fn validate_websocket_outbound_profile(
    profile: &HostWebSocketOutboundProfile,
) -> Result<()> {
    if !profile.wss_only && !profile.allow_insecure_ws_for_tests {
        anyhow::bail!(
            "outbound.websocket.wss_only=false is only supported with allow_insecure_ws_for_tests=true"
        )
    }
    if profile.max_idle_ms == 0 {
        anyhow::bail!("outbound.websocket.max_idle_ms must be greater than zero")
    }
    if profile.max_duration_ms == 0 {
        anyhow::bail!("outbound.websocket.max_duration_ms must be greater than zero")
    }
    if profile.max_frame_bytes == 0 {
        anyhow::bail!("outbound.websocket.max_frame_bytes must be greater than zero")
    }
    if profile.max_total_bytes_inbound == 0 {
        anyhow::bail!("outbound.websocket.max_total_bytes_inbound must be greater than zero")
    }
    if profile.max_total_bytes_outbound == 0 {
        anyhow::bail!("outbound.websocket.max_total_bytes_outbound must be greater than zero")
    }
    if profile.max_concurrent_connections == 0 {
        anyhow::bail!("outbound.websocket.max_concurrent_connections must be greater than zero")
    }
    if !profile.enabled {
        return Ok(());
    }
    if matches!(profile.executor, HostWebSocketOutboundExecutorKind::Live)
        && profile.allowed_hosts.is_empty()
    {
        anyhow::bail!(
            "outbound.websocket.allowed_hosts is required when websocket outbound is enabled with live executor"
        )
    }
    if profile
        .allowed_hosts
        .iter()
        .any(|host| host.trim().is_empty() || host == "*")
    {
        anyhow::bail!(
            "outbound.websocket.allowed_hosts must not contain empty hosts or wildcard hosts"
        )
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    use crate::cli::{
        Cli, Command, HostWebSocketOutboundExecutorKind, HostWebSocketOutboundProfile,
    };

    #[test]
    fn validate_websocket_outbound_profile_fails_closed_on_live_with_empty_allowed_hosts() {
        let profile = HostWebSocketOutboundProfile {
            enabled: true,
            executor: HostWebSocketOutboundExecutorKind::Live,
            allowed_hosts: Vec::new(),
            ..HostWebSocketOutboundProfile::default()
        };
        let err = validate_websocket_outbound_profile(&profile)
            .expect_err("live websocket without hosts should fail closed");
        assert!(err.to_string().contains("allowed_hosts"));
    }

    #[test]
    fn validate_websocket_outbound_profile_rejects_non_wss_when_no_test_flag() {
        let profile = HostWebSocketOutboundProfile {
            enabled: true,
            executor: HostWebSocketOutboundExecutorKind::Fake,
            wss_only: false,
            allow_insecure_ws_for_tests: false,
            ..HostWebSocketOutboundProfile::default()
        };
        let err = validate_websocket_outbound_profile(&profile)
            .expect_err("non-wss websocket without test flag should fail");
        assert!(err.to_string().contains("wss_only=false"));
    }

    #[test]
    fn parses_host_serve_paas_args() {
        let cli = Cli::try_parse_from([
            "ygg",
            "host",
            "serve",
            "--http",
            "0.0.0.0:8080",
            "--profile",
            "/data/profiles/zeabur.yaml",
            "--static-dir",
            "/app/public",
            "--data-dir",
            "/data",
            "--access-token",
            "token",
        ])
        .unwrap();

        match cli.command {
            Command::Host {
                command:
                    crate::cli::HostCommand::Serve {
                        http,
                        profile,
                        static_dir,
                        data_dir,
                        access_token,
                    },
            } => {
                assert_eq!(http, "0.0.0.0:8080".parse().unwrap());
                assert_eq!(profile, Some(PathBuf::from("/data/profiles/zeabur.yaml")));
                assert_eq!(static_dir, Some(PathBuf::from("/app/public")));
                assert_eq!(data_dir, Some(PathBuf::from("/data")));
                assert_eq!(access_token, Some("token".to_string()));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn resolve_profile_relative_event_store_under_profile_dir() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let profile_path = tmp.path().join("profiles/default.yaml");
        let event_path = resolve_profile_path(&profile_path, PathBuf::from("events.sqlite"));
        assert_eq!(event_path, tmp.path().join("profiles/events.sqlite"));
        Ok(())
    }
}

async fn serve_runtime<S>(
    http: SocketAddr,
    runtime: Arc<Runtime<S>>,
    backend_kind: &'static str,
    profile_path: Option<&Path>,
    static_dir: Option<PathBuf>,
    access_token: Option<String>,
) -> Result<()>
where
    S: EventStore,
{
    let count = if let Some(profile_path) = profile_path {
        let data_dir = profile_path
            .parent()
            .and_then(Path::parent)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let projects_dir = data_dir.join("projects");
        fs::create_dir_all(&projects_dir).with_context(|| {
            format!(
                "failed to create projects directory {}",
                projects_dir.display()
            )
        })?;
        runtime
            .config()
            .project_registry
            .load_from_projects_dir(&projects_dir)
            .with_context(|| format!("failed to load projects from {}", projects_dir.display()))?
    } else {
        let projects_dir = ygg_core::paths::projects_dir()?;
        fs::create_dir_all(&projects_dir).with_context(|| {
            format!(
                "failed to create projects directory {}",
                projects_dir.display()
            )
        })?;
        runtime
            .config()
            .project_registry
            .load_from_projects_dir(&projects_dir)
            .with_context(|| format!("failed to load projects from {}", projects_dir.display()))?
    };
    println!("  projects loaded: {count}");
    let listener = tokio::net::TcpListener::bind(http).await?;
    println!("Yggdrasil host serving http://{http}");
    println!("  event store: {backend_kind} (config redacted)");
    println!("  RPC: POST http://{http}/rpc");
    println!("  SSE: GET  http://{http}/kernel/v1/event.subscribe/:session_id");
    if let Some(static_dir) = &static_dir {
        println!("  static: GET http://{http}/ -> {}", static_dir.display());
    }
    if access_token
        .as_deref()
        .is_some_and(|token| !token.is_empty())
    {
        println!("  access token: enabled (value redacted)");
    } else {
        println!("  access token: disabled (local/dev only)");
    }
    let app = ygg_service::app_with_state(ygg_service::AppState {
        runtime,
        static_dir,
        access_token,
    });
    axum::serve(listener, app).await?;
    Ok(())
}

fn resolve_profile_path(profile_path: &std::path::Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        profile_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(path)
    }
}

pub(crate) async fn load_host_profile<S>(
    runtime: Arc<Runtime<S>>,
    profile_path: PathBuf,
) -> Result<()>
where
    S: EventStore,
{
    let raw = fs::read_to_string(&profile_path)
        .with_context(|| format!("failed to read host profile {}", profile_path.display()))?;
    let profile: HostProfile = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse host profile {}", profile_path.display()))?;
    load_profile_packages(runtime, profile, profile_path).await
}

async fn load_profile_packages<S>(
    runtime: Arc<Runtime<S>>,
    profile: HostProfile,
    profile_path: PathBuf,
) -> Result<()>
where
    S: EventStore,
{
    if let Some(title) = &profile.title {
        println!("loading host profile: {title}");
    }
    let base = profile_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    for manifest_path in profile.autoload {
        let resolved = if manifest_path.is_absolute() {
            manifest_path
        } else {
            base.join(manifest_path)
        };
        let manifest = read_manifest(resolved.clone()).await.with_context(|| {
            format!("failed to autoload package manifest {}", resolved.display())
        })?;
        let record = runtime.load_package(manifest).await?;
        println!(
            "autoloaded package: {}@{} ({:?})",
            record.id, record.version, record.state
        );
    }
    Ok(())
}

pub(crate) async fn host_stdio() -> Result<()> {
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
            Ok(request) => match runtime
                .call_protocol(&context, &request.method, request.params)
                .await
            {
                Ok(result) => ygg_runtime::ProtocolResponse {
                    id: request.id,
                    result: Some(result),
                    error: None,
                },
                Err(error) => ygg_runtime::ProtocolResponse {
                    id: request.id,
                    result: None,
                    error: Some(error),
                },
            },
            Err(error) => ygg_runtime::ProtocolResponse {
                id: "invalid".to_string(),
                result: None,
                error: Some(ygg_runtime::ProtocolError::invalid_request(
                    error.to_string(),
                )),
            },
        };
        stdout
            .write_all(serde_json::to_string(&response)?.as_bytes())
            .await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }
    Ok(())
}
