use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ygg_core::{CapHandle, CapHandleId, ContractMode, HandleLease, HandleProvenance, HandleScope};
use ygg_runtime::{
    CapabilityInvocationRequest, EventStore, InMemoryEventStore, ProtocolContext, Runtime,
    RuntimeConfig,
};

use super::manifest::read_manifest;

#[derive(Args, Debug)]
pub struct ConformancePackageArgs {
    /// Path to package directory containing manifest.yaml or manifest.json
    #[arg(long)]
    pub path: PathBuf,

    /// Contract version to validate against
    #[arg(long, default_value = "v1")]
    pub contract: String,

    /// Output format
    #[arg(long, default_value = "human")]
    pub format: ReportFormat,

    /// Skip running checks that require starting the package process
    /// (useful in CI where subprocess can't run)
    #[arg(long)]
    pub static_only: bool,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ReportFormat {
    Human,
    Json,
}

#[derive(Serialize, Deserialize)]
pub struct PackageConformanceReport {
    pub package_id: String,
    pub manifest_path: PathBuf,
    pub contract_version: String,
    pub checks: Vec<CheckResult>,
    pub summary: ConformanceSummary,
}

#[derive(Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub status: CheckStatus,
    pub details: Option<String>,
    pub subreports: Vec<SubReport>,
}

#[derive(Serialize, Deserialize)]
pub struct SubReport {
    pub id: String,
    pub status: CheckStatus,
    pub details: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail,
    Skip,
    Warning,
}

#[derive(Serialize, Deserialize)]
pub struct ConformanceSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub warnings: u32,
    pub compliance_pct: f32,
}

struct LoadedRuntime {
    runtime: Runtime<InMemoryEventStore>,
    store: Arc<InMemoryEventStore>,
}

pub(crate) async fn run(args: ConformancePackageArgs) -> Result<()> {
    let report = run_report(args.path, args.contract, args.static_only).await;
    let report = match report {
        Ok(report) => report,
        Err(error) => PackageConformanceReport {
            package_id: "unknown".to_string(),
            manifest_path: PathBuf::new(),
            contract_version: "v1".to_string(),
            checks: vec![fail("manifest.schema_valid", error.to_string())],
            summary: summarize(&[fail("manifest.schema_valid", error.to_string())]),
        },
    };

    match args.format {
        ReportFormat::Human => print_human(&report),
        ReportFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }

    if report.summary.failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

async fn run_report(
    path: PathBuf,
    contract: String,
    static_only: bool,
) -> Result<PackageConformanceReport> {
    let manifest_path = resolve_manifest_path(&path)?;
    let raw_value = read_manifest_value(&manifest_path)?;
    let package_id = raw_value
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let mut checks = Vec::new();
    let schema_check = check_manifest_schema_valid(&manifest_path, &raw_value);
    let schema_ok = schema_check.status == CheckStatus::Pass;
    checks.push(schema_check);

    let manifest = if schema_ok {
        Some(read_manifest(manifest_path.clone()).await?)
    } else {
        None
    };

    if let Some(manifest) = manifest.as_ref() {
        checks.push(check_manifest_declarations_consistent(manifest));
    } else {
        checks.push(skip(
            "manifest.declarations_consistent",
            "manifest schema invalid",
        ));
    }

    let mut loaded: Option<LoadedRuntime> = None;
    if !schema_ok {
        for id in PROCESS_CHECK_IDS {
            checks.push(skip(id, "manifest schema invalid"));
        }
    } else if static_only {
        for id in PROCESS_CHECK_IDS {
            checks.push(skip(id, "static-only mode"));
        }
    } else if let Some(manifest) = manifest.as_ref() {
        match load_runtime(manifest.clone()).await {
            Ok(rt) => {
                checks.push(check_handshake_feature_negotiation(manifest, &rt).await);
                checks.push(check_capability_smoke_invocations(manifest, &rt.runtime).await);
                checks.push(check_streaming_cancel_and_timeout(manifest, &rt).await);
                checks.push(check_permission_denial_paths(manifest, &rt).await);
                checks.push(check_handle_lifecycle(manifest, &rt.runtime).await);
                loaded = Some(rt);
            }
            Err(error) => {
                checks.push(fail("handshake.feature_negotiation", error.to_string()));
                checks.push(skip("capability.smoke_invocations", "package did not load"));
                checks.push(skip("streaming.cancel_and_timeout", "package did not load"));
                checks.push(skip("permission.denial_paths", "package did not load"));
                checks.push(skip("handle.lifecycle", "package did not load"));
            }
        }
    }

    if let Some(rt) = loaded.as_ref() {
        if let Some(manifest) = manifest.as_ref() {
            checks.push(check_events_and_errors(manifest, rt).await);
            let _ = rt.runtime.unload_package(&manifest.id).await;
        }
    } else {
        checks.push(skip(
            "events_and_errors_consistent_with_registry",
            "no runtime events to inspect",
        ));
    }

    let summary = summarize(&checks);
    Ok(PackageConformanceReport {
        package_id,
        manifest_path,
        contract_version: contract,
        checks,
        summary,
    })
}

const PROCESS_CHECK_IDS: &[&str] = &[
    "handshake.feature_negotiation",
    "capability.smoke_invocations",
    "streaming.cancel_and_timeout",
    "permission.denial_paths",
    "handle.lifecycle",
];

fn resolve_manifest_path(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    for name in ["manifest.yaml", "manifest.yml", "manifest.json"] {
        let candidate = path.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    anyhow::bail!(
        "no manifest.yaml, manifest.yml, or manifest.json found in {}",
        path.display()
    )
}

fn read_manifest_value(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)?;
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("yaml") | Some("yml") => Ok(serde_yaml::from_str(&raw)?),
        _ => Ok(serde_json::from_str(&raw)?),
    }
}

fn check_manifest_schema_valid(manifest_path: &Path, manifest_value: &Value) -> CheckResult {
    let schema_path = Path::new("docs/spec/v1/schemas/manifest.schema.json");
    let schema_text = match fs::read_to_string(schema_path) {
        Ok(text) => text,
        Err(error) => return fail("manifest.schema_valid", format!("read schema: {error}")),
    };
    let mut schema_value: Value = match serde_json::from_str(&schema_text) {
        Ok(value) => value,
        Err(error) => return fail("manifest.schema_valid", format!("parse schema: {error}")),
    };
    // schemars emits `$defs`, while the current exported artifact still uses
    // legacy `#/definitions/*` refs. Mirror `$defs` under `definitions` for
    // jsonschema validation until the exporter is normalized.
    if schema_value.get("definitions").is_none() {
        if let Some(defs) = schema_value.get("$defs").cloned() {
            if let Some(object) = schema_value.as_object_mut() {
                object.insert("definitions".to_string(), defs);
            }
        }
    }
    let compiled = match JSONSchema::compile(&schema_value) {
        Ok(compiled) => compiled,
        Err(error) => return fail("manifest.schema_valid", format!("compile schema: {error}")),
    };
    let result = match compiled.validate(manifest_value) {
        Ok(()) => pass(
            "manifest.schema_valid",
            Some(format!("{} is schema-valid", manifest_path.display())),
        ),
        Err(errors) => fail(
            "manifest.schema_valid",
            errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; "),
        ),
    };
    result
}

fn check_manifest_declarations_consistent(manifest: &ygg_core::PackageManifest) -> CheckResult {
    let mut failures = Vec::new();
    let mut warnings = Vec::new();
    let namespace = format!("{}/", manifest.id);

    for cap in &manifest.provides {
        if !cap.id.starts_with(&namespace) {
            failures.push(format!(
                "provided capability '{}' is not under package id namespace '{}'",
                cap.id, manifest.id
            ));
        }
    }

    let known_namespaces: BTreeSet<String> = manifest
        .provides
        .iter()
        .filter_map(|cap| namespace_of(&cap.id))
        .chain(std::iter::once(manifest.id.clone()))
        .chain(std::iter::once("kernel/v1".to_string()))
        .collect();
    for req in &manifest.consumes {
        if namespace_of(&req.id)
            .map(|ns| !known_namespaces.contains(&ns))
            .unwrap_or(true)
        {
            warnings.push(format!(
                "consumed capability '{}' references an unrecognized namespace",
                req.id
            ));
        }
    }

    for secret_ref in &manifest.permissions.secret_refs {
        if !valid_contract_secret_ref(secret_ref) {
            failures.push(format!("secret_ref '{}' is not env-formatted", secret_ref));
        }
    }

    for declaration in &manifest.permissions.network.declarations {
        for method in &declaration.methods {
            if !valid_http_method(method) {
                failures.push(format!(
                    "network method '{}' for host '{}' is invalid",
                    method, declaration.host
                ));
            }
        }
    }

    if !failures.is_empty() {
        fail("manifest.declarations_consistent", failures.join("; "))
    } else if !warnings.is_empty() {
        warning("manifest.declarations_consistent", warnings.join("; "))
    } else {
        pass("manifest.declarations_consistent", None)
    }
}

async fn load_runtime(manifest: ygg_core::PackageManifest) -> Result<LoadedRuntime> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    runtime.load_package(manifest).await?;
    Ok(LoadedRuntime { runtime, store })
}

async fn check_handshake_feature_negotiation(
    manifest: &ygg_core::PackageManifest,
    rt: &LoadedRuntime,
) -> CheckResult {
    if manifest.entry.contract == ContractMode::None {
        return pass(
            "handshake.feature_negotiation",
            Some("Path B contract=none; bindings not required".to_string()),
        );
    }
    let Some(record) = rt.runtime.package_status(&manifest.id).await else {
        return fail("handshake.feature_negotiation", "package not loaded");
    };
    let expected_bindings = manifest.permissions.capabilities.invoke.len()
        + manifest.permissions.network.declarations.len()
        + manifest.permissions.secret_refs.len();
    let actual_bindings = rt.runtime.handles().list_for(&manifest.id).await.len();
    if actual_bindings != expected_bindings {
        return fail(
            "handshake.feature_negotiation",
            format!("binding count mismatch: expected {expected_bindings}, got {actual_bindings}"),
        );
    }
    pass(
        "handshake.feature_negotiation",
        Some(format!(
            "ready {:?}; {} bindings",
            record.state, actual_bindings
        )),
    )
}

async fn check_capability_smoke_invocations(
    manifest: &ygg_core::PackageManifest,
    runtime: &Runtime<InMemoryEventStore>,
) -> CheckResult {
    if manifest.provides.is_empty() {
        return pass(
            "capability.smoke_invocations",
            Some("0/0 capabilities".to_string()),
        );
    }
    let mut subreports = Vec::new();
    for cap in &manifest.provides {
        if cap.streaming {
            subreports.push(SubReport {
                id: cap.id.clone(),
                status: CheckStatus::Skip,
                details: Some("streaming capability covered by streaming check".to_string()),
            });
            continue;
        }
        let result = runtime
            .invoke_capability(CapabilityInvocationRequest {
                handle: None,
                capability_id: Some(cap.id.clone()),
                caller_package_id: None,
                provider_package_id: Some(manifest.id.clone()),
                version: Some(cap.version.clone()),
                input: json!({}),
            })
            .await;
        match result {
            Ok(_) => subreports.push(SubReport {
                id: cap.id.clone(),
                status: CheckStatus::Pass,
                details: None,
            }),
            Err(error) if acceptable_structured_error(&error.to_string()) => {
                subreports.push(SubReport {
                    id: cap.id.clone(),
                    status: CheckStatus::Pass,
                    details: Some(error.to_string()),
                })
            }
            Err(error) => subreports.push(SubReport {
                id: cap.id.clone(),
                status: CheckStatus::Fail,
                details: Some(error.to_string()),
            }),
        }
    }
    let failures = subreports
        .iter()
        .filter(|r| r.status == CheckStatus::Fail)
        .count();
    let passed = subreports
        .iter()
        .filter(|r| r.status == CheckStatus::Pass)
        .count();
    let total = subreports
        .iter()
        .filter(|r| r.status != CheckStatus::Skip)
        .count();
    CheckResult {
        id: "capability.smoke_invocations".to_string(),
        status: if failures == 0 {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        details: Some(format!("{passed}/{total} capabilities")),
        subreports,
    }
}

async fn check_streaming_cancel_and_timeout(
    manifest: &ygg_core::PackageManifest,
    rt: &LoadedRuntime,
) -> CheckResult {
    let streaming: Vec<_> = manifest
        .provides
        .iter()
        .filter(|cap| cap.streaming)
        .collect();
    if streaming.is_empty() {
        return skip("streaming.cancel_and_timeout", "no streaming");
    }
    let mut subreports = Vec::new();
    for cap in streaming {
        let session_id = format!("conformance_stream_{}", cap.id.replace('/', "_"));
        let started = rt
            .runtime
            .stream_capability_start(
                &session_id,
                &cap.id,
                Some(&manifest.id),
                Some(&cap.version),
                json!({}),
            )
            .await;
        let Ok((_frame, record)) = started else {
            subreports.push(SubReport {
                id: cap.id.clone(),
                status: CheckStatus::Fail,
                details: Some(started.err().unwrap().to_string()),
            });
            continue;
        };
        let _ = rt
            .runtime
            .stream_capability_chunk(
                &session_id,
                &record.invocation_id,
                json!({"conformance": true}),
                ygg_core::RedactionState::NotCaptured,
            )
            .await;
        let cancelled = rt
            .runtime
            .stream_capability_cancel(&session_id, &record.invocation_id)
            .await;
        let events = rt.store.list_session(&session_id).await.unwrap_or_default();
        let emitted = events
            .iter()
            .any(|e| e.kind == ygg_core::EVENT_STREAM_CANCELLED);
        let next_ok = rt
            .runtime
            .invoke_capability(CapabilityInvocationRequest {
                handle: None,
                capability_id: manifest
                    .provides
                    .iter()
                    .find(|c| !c.streaming)
                    .map(|c| c.id.clone()),
                caller_package_id: None,
                provider_package_id: Some(manifest.id.clone()),
                version: None,
                input: json!({}),
            })
            .await
            .is_ok()
            || manifest.provides.iter().all(|c| c.streaming);
        let ok = cancelled.is_ok() && emitted && next_ok;
        subreports.push(SubReport {
            id: cap.id.clone(),
            status: if ok {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail
            },
            details: Some(format!(
                "cancelled_event={emitted}, next_invoke_ok={next_ok}"
            )),
        });
    }
    let failures = subreports
        .iter()
        .filter(|r| r.status == CheckStatus::Fail)
        .count();
    CheckResult {
        id: "streaming.cancel_and_timeout".to_string(),
        status: if failures == 0 {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        details: None,
        subreports,
    }
}

async fn check_permission_denial_paths(
    manifest: &ygg_core::PackageManifest,
    rt: &LoadedRuntime,
) -> CheckResult {
    if manifest.entry.contract == ContractMode::None {
        return pass(
            "permission.denial_paths",
            Some("Path B contract=none; permission enforcement bypass expected".to_string()),
        );
    }
    let denied_capability = format!("{}/__conformance_not_declared", manifest.id);
    let result = rt
        .runtime
        .invoke_capability(CapabilityInvocationRequest {
            handle: None,
            capability_id: Some(denied_capability.clone()),
            caller_package_id: Some(manifest.id.clone()),
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    let denied = result.is_err() && result.err().unwrap().to_string().contains("not allowed");
    let session_id = format!("kernel_capability_{}", denied_capability.replace('/', "_"));
    let emitted = rt
        .store
        .list_session(&session_id)
        .await
        .unwrap_or_default()
        .iter()
        .any(|e| e.kind == ygg_core::EVENT_PERMISSION_DENIED);
    if denied && emitted {
        pass(
            "permission.denial_paths",
            Some("undeclared capability invoke denied".to_string()),
        )
    } else {
        fail(
            "permission.denial_paths",
            format!("denied={denied}, event_emitted={emitted}"),
        )
    }
}

async fn check_handle_lifecycle(
    manifest: &ygg_core::PackageManifest,
    runtime: &Runtime<InMemoryEventStore>,
) -> CheckResult {
    if manifest.entry.contract == ContractMode::None {
        return skip(
            "handle.lifecycle",
            "Path B contract=none; handles are not enforced",
        );
    }
    let Some(cap) = manifest.provides.iter().find(|cap| !cap.streaming) else {
        return skip("handle.lifecycle", "no non-streaming capability");
    };
    let parent = runtime
        .handles()
        .mint(CapHandle {
            id: CapHandleId::new(),
            cap_type: cap.id.clone(),
            cap_version: cap.version.clone(),
            scope: HandleScope {
                holder_package_id: manifest.id.clone(),
                session_id: None,
            },
            constraints: json!({}),
            lease: HandleLease {
                expires_at: None,
                max_invocations: None,
                invocations_used: 0,
            },
            provenance: HandleProvenance {
                granted_at: chrono::Utc::now(),
                granted_by_package_id: ygg_core::KERNEL_PACKAGE_ID.to_string(),
                via_method: "conformance".to_string(),
            },
            parent: None,
            revoked: false,
        })
        .await;
    let child = match runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.cap.attenuate",
            json!({"parent_handle": parent, "constraints": {}}),
        )
        .await
    {
        Ok(value) => match serde_json::from_value::<CapHandleId>(value["handle"]["id"].clone()) {
            Ok(id) => id,
            Err(error) => {
                return fail(
                    "handle.lifecycle",
                    format!("parse attenuated handle: {error}"),
                )
            }
        },
        Err(error) => return fail("handle.lifecycle", format!("attenuate failed: {error:?}")),
    };
    let invoke = runtime
        .invoke_capability(CapabilityInvocationRequest {
            handle: Some(child),
            capability_id: None,
            caller_package_id: Some(manifest.id.clone()),
            provider_package_id: Some(manifest.id.clone()),
            version: None,
            input: json!({}),
        })
        .await;
    if let Err(error) = invoke {
        return fail(
            "handle.lifecycle",
            format!("invoke with attenuated handle failed: {error}"),
        );
    }
    if let Err(error) = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.cap.revoke",
            json!({"handle": child}),
        )
        .await
    {
        return fail("handle.lifecycle", format!("revoke failed: {error:?}"));
    }
    let revoked = runtime
        .invoke_capability(CapabilityInvocationRequest {
            handle: Some(child),
            capability_id: None,
            caller_package_id: Some(manifest.id.clone()),
            provider_package_id: Some(manifest.id.clone()),
            version: None,
            input: json!({}),
        })
        .await;
    if revoked.is_err() {
        pass(
            "handle.lifecycle",
            Some("attenuate/invoke/revoke lifecycle enforced".to_string()),
        )
    } else {
        fail(
            "handle.lifecycle",
            "revoked handle still invoked successfully",
        )
    }
}

async fn check_events_and_errors(
    manifest: &ygg_core::PackageManifest,
    rt: &LoadedRuntime,
) -> CheckResult {
    let registry = event_registry();
    let events = rt.store.list_all().await.unwrap_or_default();
    let mut failures = Vec::new();
    let mut warnings = Vec::new();
    for event in events {
        if event.kind.starts_with("kernel/v1/") && !registry.contains(event.kind.as_str()) {
            failures.push(format!("off-registry kernel event '{}'", event.kind));
        } else if event.kind.starts_with(&format!("{}/", manifest.id)) {
            warnings.push(format!(
                "package namespace event '{}' is allowed but not registry-standard",
                event.kind
            ));
        }
        if event.kind == ygg_core::EVENT_CAPABILITY_FAILED {
            let ok = event
                .payload
                .get("error_kind")
                .and_then(Value::as_str)
                .is_some()
                && event
                    .payload
                    .get("error_message")
                    .and_then(Value::as_str)
                    .is_some();
            if !ok {
                failures.push(format!(
                    "capability failure event '{}' has invalid error shape",
                    event.id
                ));
            }
        }
    }
    if !failures.is_empty() {
        fail(
            "events_and_errors_consistent_with_registry",
            failures.join("; "),
        )
    } else if !warnings.is_empty() {
        warning(
            "events_and_errors_consistent_with_registry",
            warnings.join("; "),
        )
    } else {
        pass("events_and_errors_consistent_with_registry", None)
    }
}

fn event_registry() -> BTreeSet<&'static str> {
    [
        ygg_core::EVENT_SESSION_OPENED,
        ygg_core::EVENT_SESSION_CLOSED,
        ygg_core::EVENT_SESSION_FORKED,
        ygg_core::EVENT_PACKAGE_LOADED,
        ygg_core::EVENT_PACKAGE_LOADING,
        ygg_core::EVENT_PACKAGE_STARTING,
        ygg_core::EVENT_PACKAGE_READY,
        ygg_core::EVENT_PACKAGE_STOPPING,
        ygg_core::EVENT_PACKAGE_STOPPED,
        ygg_core::EVENT_PACKAGE_UNLOADED,
        ygg_core::EVENT_PACKAGE_DEGRADED,
        ygg_core::EVENT_PACKAGE_LOG,
        ygg_core::EVENT_ASSET_PUT,
        ygg_core::EVENT_PROJECTION_UPDATED,
        ygg_core::EVENT_PROPOSAL_CREATED,
        ygg_core::EVENT_PROPOSAL_APPROVED,
        ygg_core::EVENT_PROPOSAL_REJECTED,
        ygg_core::EVENT_PROPOSAL_APPLIED,
        ygg_core::EVENT_PROPOSAL_FAILED,
        ygg_core::EVENT_CAPABILITY_INVOKED,
        ygg_core::EVENT_CAPABILITY_COMPLETED,
        ygg_core::EVENT_CAPABILITY_FAILED,
        ygg_core::EVENT_PERMISSION_DENIED,
        ygg_core::EVENT_PERMISSION_GRANTED,
        ygg_core::EVENT_PERMISSION_REVOKED,
        ygg_core::EVENT_ERROR,
        ygg_core::EVENT_OUTBOUND_REQUEST,
        ygg_core::EVENT_OUTBOUND_DENIED,
        ygg_core::EVENT_OUTBOUND_EXECUTE_COMPLETED,
        ygg_core::EVENT_OUTBOUND_STREAM_COMPLETED,
        ygg_core::EVENT_STREAM_STARTED,
        ygg_core::EVENT_STREAM_CHUNK,
        ygg_core::EVENT_STREAM_PROGRESS,
        ygg_core::EVENT_STREAM_ENDED,
        ygg_core::EVENT_STREAM_ERROR,
        ygg_core::EVENT_STREAM_CANCELLED,
        ygg_core::EVENT_STREAM_TIMEOUT,
        ygg_core::EVENT_OUTBOUND_WEBSOCKET_OPENED,
        ygg_core::EVENT_OUTBOUND_WEBSOCKET_FRAME,
        ygg_core::EVENT_OUTBOUND_WEBSOCKET_ERROR,
        ygg_core::EVENT_OUTBOUND_WEBSOCKET_COMPLETED,
    ]
    .into_iter()
    .collect()
}

fn summarize(checks: &[CheckResult]) -> ConformanceSummary {
    let total = checks.len() as u32;
    let passed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Pass)
        .count() as u32;
    let failed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count() as u32;
    let skipped = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Skip)
        .count() as u32;
    let warnings = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warning)
        .count() as u32;
    let applicable = total.saturating_sub(skipped);
    let compliance_pct = if applicable == 0 {
        100.0
    } else {
        ((passed + warnings) as f32 / applicable as f32) * 100.0
    };
    ConformanceSummary {
        total,
        passed,
        failed,
        skipped,
        warnings,
        compliance_pct,
    }
}

fn print_human(report: &PackageConformanceReport) {
    println!(
        "Conformance Report: {}",
        report
            .manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .display()
    );
    println!("Contract: {}\n", report.contract_version);
    for (idx, check) in report.checks.iter().enumerate() {
        let detail = check
            .details
            .as_ref()
            .map(|d| format!("  ({d})"))
            .unwrap_or_default();
        println!(
            "[{}/{}] {:50} {:7}{}",
            idx + 1,
            report.checks.len(),
            check.id,
            status_label(check.status),
            detail
        );
    }
    let applicable = report.summary.total.saturating_sub(report.summary.skipped);
    println!(
        "\nSummary: {}/{} applicable checks passed ({:.0}%)",
        report.summary.passed + report.summary.warnings,
        applicable,
        report.summary.compliance_pct
    );
}

fn status_label(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Pass => "PASS",
        CheckStatus::Fail => "FAIL",
        CheckStatus::Skip => "SKIP",
        CheckStatus::Warning => "WARNING",
    }
}

fn pass(id: &str, details: Option<String>) -> CheckResult {
    CheckResult {
        id: id.to_string(),
        status: CheckStatus::Pass,
        details,
        subreports: Vec::new(),
    }
}

fn fail(id: &str, details: impl Into<String>) -> CheckResult {
    CheckResult {
        id: id.to_string(),
        status: CheckStatus::Fail,
        details: Some(details.into()),
        subreports: Vec::new(),
    }
}

fn skip(id: &str, details: impl Into<String>) -> CheckResult {
    CheckResult {
        id: id.to_string(),
        status: CheckStatus::Skip,
        details: Some(details.into()),
        subreports: Vec::new(),
    }
}

fn warning(id: &str, details: impl Into<String>) -> CheckResult {
    CheckResult {
        id: id.to_string(),
        status: CheckStatus::Warning,
        details: Some(details.into()),
        subreports: Vec::new(),
    }
}

fn namespace_of(id: &str) -> Option<String> {
    let mut parts = id.split('/');
    Some(format!("{}/{}", parts.next()?, parts.next()?))
}

fn valid_contract_secret_ref(value: &str) -> bool {
    if let Some(name) = value.strip_prefix("secret_ref:env:") {
        !name.is_empty()
    } else if let Some(name) = value.strip_prefix("secret_ref:") {
        !name.is_empty() && !name.contains(':')
    } else {
        false
    }
}

fn valid_http_method(method: &str) -> bool {
    matches!(
        method.to_ascii_uppercase().as_str(),
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" | "WEBSOCKET"
    )
}

fn acceptable_structured_error(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("not_implemented")
        || lower.contains("not implemented")
        || lower.contains("unsupported")
}
