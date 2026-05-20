use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use serde::Serialize;
use serde_json::json;
use ygg_runtime::{
    AppendEventRequest, CapabilityInvocationRequest, EventStore, InMemoryEventStore,
    OpenSessionRequest, Runtime, RuntimeConfig,
};

use crate::cli::BaselineFormat;
use crate::commands::manifest::read_manifest;

// ── Scenario result ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ScenarioResult {
    scenario_id: &'static str,
    iterations: u32,
    total_ms: f64,
    avg_ms: f64,
    min_ms: f64,
    max_ms: f64,
    status: &'static str,
    notes: Option<String>,
}

// ── Helpers ────────────────────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn manifest_path(rel: &str) -> PathBuf {
    workspace_root().join(rel)
}

fn build_result(
    scenario_id: &'static str,
    iterations: u32,
    durations: &[f64],
) -> ScenarioResult {
    let total_ms: f64 = durations.iter().sum();
    let avg_ms = total_ms / durations.len() as f64;
    let min_ms = durations.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ms = durations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    ScenarioResult {
        scenario_id,
        iterations,
        total_ms,
        avg_ms,
        min_ms,
        max_ms,
        status: "ok",
        notes: None,
    }
}

fn error_result(scenario_id: &'static str, iterations: u32, err: impl std::fmt::Display) -> ScenarioResult {
    ScenarioResult {
        scenario_id,
        iterations,
        total_ms: 0.0,
        avg_ms: 0.0,
        min_ms: 0.0,
        max_ms: 0.0,
        status: "error",
        notes: Some(format!("error: {err}")),
    }
}

fn skipped_result(scenario_id: &'static str, iterations: u32, reason: &str) -> ScenarioResult {
    ScenarioResult {
        scenario_id,
        iterations,
        total_ms: 0.0,
        avg_ms: 0.0,
        min_ms: 0.0,
        max_ms: 0.0,
        status: "skipped",
        notes: Some(reason.to_string()),
    }
}

// ── Scenarios ──────────────────────────────────────────────────────────

async fn scenario_inproc_echo_invoke(iterations: u32) -> ScenarioResult {
    let manifest_path = manifest_path("examples/packages/echo-rust-inproc/manifest.yaml");
    let manifest = match read_manifest(manifest_path).await {
        Ok(m) => m,
        Err(e) => return error_result("inproc_echo_invoke", iterations, e),
    };
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    if let Err(e) = runtime.load_package(manifest).await {
        return error_result("inproc_echo_invoke", iterations, e);
    }

    let mut durations = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        let start = Instant::now();
        match runtime
            .invoke_capability(CapabilityInvocationRequest {
                capability_id: "example/echo-rust-inproc/echo".to_string(),
                caller_package_id: None,
                provider_package_id: None,
                version: None,
                input: json!({"baseline": true}),
            })
            .await
        {
            Ok(_) => durations.push(start.elapsed().as_secs_f64() * 1000.0),
            Err(e) => return error_result("inproc_echo_invoke", iterations, e),
        }
    }
    build_result("inproc_echo_invoke", iterations, &durations)
}

async fn scenario_official_capability_invoke(iterations: u32) -> ScenarioResult {
    let manifest_path = manifest_path("packages/official/composition-lab/manifest.yaml");
    let manifest = match read_manifest(manifest_path).await {
        Ok(m) => m,
        Err(e) => return error_result("official_capability_invoke", iterations, e),
    };
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    if let Err(e) = runtime.load_package(manifest).await {
        return error_result("official_capability_invoke", iterations, e);
    }

    let mut durations = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        let start = Instant::now();
        match runtime
            .invoke_capability(CapabilityInvocationRequest {
                capability_id: "official/composition-lab/describe".to_string(),
                caller_package_id: None,
                provider_package_id: None,
                version: None,
                input: json!({}),
            })
            .await
        {
            Ok(_) => durations.push(start.elapsed().as_secs_f64() * 1000.0),
            Err(e) => return error_result("official_capability_invoke", iterations, e),
        }
    }
    let mut r = build_result("official_capability_invoke", iterations, &durations);
    r.notes = Some("official/composition-lab/describe".to_string());
    r
}

async fn scenario_event_store_append_list_range(iterations: u32) -> ScenarioResult {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());

    let manifest_path = manifest_path("examples/packages/echo-rust-inproc/manifest.yaml");
    let manifest = match read_manifest(manifest_path).await {
        Ok(m) => m,
        Err(e) => return error_result("event_store_append_list_range", iterations, e),
    };
    if let Err(e) = runtime.load_package(manifest).await {
        return error_result("event_store_append_list_range", iterations, e);
    }
    let session = match runtime.open_session(OpenSessionRequest::default()).await {
        Ok(s) => s,
        Err(e) => return error_result("event_store_append_list_range", iterations, e),
    };

    let event_count: u32 = 100;
    let mut durations = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = Instant::now();

        // Append 100 events
        for i in 0..event_count {
            if let Err(e) = runtime
                .append_event(AppendEventRequest {
                    session_id: session.id.clone(),
                    writer_package_id: "example/echo-rust-inproc".to_string(),
                    kind: "example/echo-rust-inproc/baseline.event".to_string(),
                    payload: json!({"i": i}),
                    metadata: json!({}),
                })
                .await
            {
                return error_result("event_store_append_list_range", iterations, e);
            }
        }

        // List all
        let events = match store.list_session(&session.id).await {
            Ok(v) => v,
            Err(e) => return error_result("event_store_append_list_range", iterations, e),
        };
        if events.len() < event_count as usize {
            return error_result(
                "event_store_append_list_range",
                iterations,
                "list_session returned too few events",
            );
        }

        // Range query (last 10)
        let range_after = if events.len() > 10 {
            (events.len() - 10) as u64
        } else {
            0
        };
        let range = match store
            .list_session_range(&session.id, Some(range_after), Some(10))
            .await
        {
            Ok(v) => v,
            Err(e) => return error_result("event_store_append_list_range", iterations, e),
        };
        if range.is_empty() {
            return error_result(
                "event_store_append_list_range",
                iterations,
                "range query returned empty",
            );
        }

        durations.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    let mut r = build_result("event_store_append_list_range", iterations, &durations);
    r.notes = Some(format!("{event_count} events per iteration"));
    r
}

async fn scenario_composition_check(iterations: u32) -> ScenarioResult {
    let composition_path =
        manifest_path("examples/compositions/playable-seed-replacement/composition.yaml");
    let raw = match fs::read_to_string(&composition_path) {
        Ok(r) => r,
        Err(e) => return error_result("composition_check", iterations, e),
    };
    let composition: crate::cli::CompositionDescriptor = match composition_path
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("yaml") | Some("yml") => match serde_yaml::from_str(&raw) {
            Ok(c) => c,
            Err(e) => return error_result("composition_check", iterations, e),
        },
        _ => match serde_json::from_str(&raw) {
            Ok(c) => c,
            Err(e) => return error_result("composition_check", iterations, e),
        },
    };

    let base = composition_path.parent().unwrap_or_else(|| std::path::Path::new("."));

    let mut durations = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        let start = Instant::now();

        // Validate fields
        if composition.id.trim().is_empty() {
            return error_result("composition_check", iterations, "composition id empty");
        }
        if composition.entry_surface_id.trim().is_empty() {
            return error_result("composition_check", iterations, "entry surface empty");
        }

        // Load required packages
        for pkg_path in &composition.packages {
            let resolved = if pkg_path.is_absolute() {
                pkg_path.clone()
            } else {
                base.join(pkg_path)
            };
            match read_manifest(resolved).await {
                Ok(m) => {
                    if let Err(e) = m.validate_basic() {
                        return error_result("composition_check", iterations, e);
                    }
                }
                Err(e) => return error_result("composition_check", iterations, e),
            }
        }

        durations.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    let mut r = build_result("composition_check", iterations, &durations);
    r.notes = Some("playable-seed-replacement".to_string());
    r
}

async fn scenario_profile_load(iterations: u32) -> ScenarioResult {
    let profile_path = manifest_path("profiles/forge-alpha.yaml");

    let mut durations = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        let start = Instant::now();
        let raw = match fs::read_to_string(&profile_path) {
            Ok(r) => r,
            Err(e) => return error_result("profile_load", iterations, e),
        };
        let _profile: crate::cli::HostProfile = match serde_yaml::from_str(&raw) {
            Ok(p) => p,
            Err(e) => return error_result("profile_load", iterations, e),
        };
        durations.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    let mut r = build_result("profile_load", iterations, &durations);
    r.notes = Some("forge-alpha.yaml parse".to_string());
    r
}

async fn scenario_subprocess_echo_invoke(iterations: u32) -> ScenarioResult {
    // Subprocess packages require Python which may not be available in CI.
    // Mark as skipped; subprocess echo will be measured in P1/P3 with
    // explicit environment checks.
    let manifest_path = manifest_path("examples/packages/echo-subprocess-python/manifest.yaml");
    let manifest = match read_manifest(manifest_path).await {
        Ok(m) => m,
        Err(e) => {
            return skipped_result(
                "subprocess_echo_invoke",
                iterations,
                &format!("manifest load failed: {e}; subprocess echo deferred to P1/P3"),
            );
        }
    };

    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());

    match runtime.load_package(manifest).await {
        Ok(_) => {}
        Err(e) => {
            return skipped_result(
                "subprocess_echo_invoke",
                iterations,
                &format!("subprocess start failed: {e}; deferred to P1/P3"),
            );
        }
    }

    let mut durations = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        let start = Instant::now();
        match runtime
            .invoke_capability(CapabilityInvocationRequest {
                capability_id: "example/echo-subprocess-python/echo".to_string(),
                caller_package_id: None,
                provider_package_id: None,
                version: None,
                input: json!({"baseline": true}),
            })
            .await
        {
            Ok(_) => durations.push(start.elapsed().as_secs_f64() * 1000.0),
            Err(e) => return error_result("subprocess_echo_invoke", iterations, e),
        }
    }
    build_result("subprocess_echo_invoke", iterations, &durations)
}

// ── Entry point ────────────────────────────────────────────────────────

pub(crate) async fn perf_baseline(iterations: u32, format: BaselineFormat) -> Result<()> {
    if iterations == 0 {
        anyhow::bail!("--iterations must be greater than 0");
    }

    if matches!(format, BaselineFormat::Text) {
        println!("ygg perf baseline — {} iterations per scenario", iterations);
        println!();
    }

    let mut results: Vec<ScenarioResult> = Vec::new();

    // 1. Inproc echo invoke
    results.push(scenario_inproc_echo_invoke(iterations).await);

    // 2. Official capability invoke
    results.push(scenario_official_capability_invoke(iterations).await);

    // 3. Event store append/list/range
    results.push(scenario_event_store_append_list_range(iterations).await);

    // 4. Composition check
    results.push(scenario_composition_check(iterations).await);

    // 5. Profile load
    results.push(scenario_profile_load(iterations).await);

    // 6. Subprocess echo (may skip)
    results.push(scenario_subprocess_echo_invoke(iterations).await);

    match format {
        BaselineFormat::Text => {
            println!(
                "{:<30} {:>10} {:>10} {:>10} {:>10} {:>8}  {}",
                "scenario", "iterations", "total_ms", "avg_ms", "min_ms", "max_ms", "status"
            );
            println!("{}", "-".repeat(90));
            for r in &results {
                let notes = r
                    .notes
                    .as_ref()
                    .map(|n| format!(" [{n}]"))
                    .unwrap_or_default();
                println!(
                    "{:<30} {:>10} {:>10.2} {:>10.3} {:>10.3} {:>8.3}  {}{}",
                    r.scenario_id,
                    r.iterations,
                    r.total_ms,
                    r.avg_ms,
                    r.min_ms,
                    r.max_ms,
                    r.status,
                    notes,
                );
            }
        }
        BaselineFormat::Json => {
            let ok_count = results.iter().filter(|r| r.status == "ok").count();
            let skip_count = results
                .iter()
                .filter(|r| r.status == "skipped")
                .count();
            let err_count = results.iter().filter(|r| r.status == "error").count();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "baseline": results,
                    "meta": {
                        "iterations": iterations,
                        "tool": "ygg perf baseline",
                        "version": "0.1.0",
                        "note": "developer-machine reference, not CI budget; no-network/deterministic",
                        "ok_count": ok_count,
                        "skipped_count": skip_count,
                        "error_count": err_count,
                    }
                }))?
            );
        }
    }

    let ok_count = results.iter().filter(|r| r.status == "ok").count();
    let skip_count = results
        .iter()
        .filter(|r| r.status == "skipped")
        .count();
    let err_count = results.iter().filter(|r| r.status == "error").count();

    if matches!(format, BaselineFormat::Text) {
        println!();
        println!(
            "baseline: {} ok, {} skipped, {} error ({} scenarios)",
            ok_count, skip_count, err_count, results.len()
        );
    }

    if err_count > 0 {
        anyhow::bail!("baseline had errors");
    }
    Ok(())
}
