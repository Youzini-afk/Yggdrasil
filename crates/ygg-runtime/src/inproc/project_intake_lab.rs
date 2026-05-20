//! Handler for `official/project-intake-lab` capabilities.
//!
//! External Project Operating Plane Alpha Phase E1 — Project Intake Lab.
//!
//! Static project intake for external project refs. No clone, no install,
//! no run, no network, no filesystem access, no shell, no outbound.
//!
//! Capabilities:
//! - describe_intake_contract: describe the intake lab contract
//! - inspect_external_project_ref: classify source ref (git/npm/local/archive/unknown)
//! - detect_project_stack_from_metadata: detect stack from metadata (node/rust/python/static/unknown)
//! - draft_workspace_plan: produce plan-only workspace plan
//! - draft_security_risk_summary: produce security risk summary
//! - list_candidate_entrypoints: list candidate entrypoints with risk annotations
//! - draft_adapter_plan: produce plan-only adapter plan
//!
//! Safety:
//! - Raw secret blocking (delegated to shared safety module)
//! - Unsafe local path rejection (path traversal, home path, absolute sensitive paths)
//! - No kernel.project/workspace/git/npm/deploy/ide namespace references
//! - No filesystem reads, no shell, no outbound, no execution

use serde_json::Value;

use super::safety;
use super::InprocInvocation;

const PACKAGE_ID: &str = "official/project-intake-lab";

// ---------------------------------------------------------------------------
// Source kinds
// ---------------------------------------------------------------------------

const SOURCE_KINDS: &[&str] = &["git", "npm", "local", "archive", "unknown"];

// ---------------------------------------------------------------------------
// Stack kinds
// ---------------------------------------------------------------------------

const STACK_KINDS: &[&str] = &["node", "rust", "python", "static", "unknown"];

// ---------------------------------------------------------------------------
// Metadata kinds
// ---------------------------------------------------------------------------

const METADATA_KINDS: &[&str] = &["package_json", "readme", "cargo_toml", "pyproject", "files"];

// ---------------------------------------------------------------------------
// NPM lifecycle scripts that execute code
// ---------------------------------------------------------------------------

const NPM_LIFECYCLE_SCRIPTS: &[&str] = &[
    "preinstall",
    "install",
    "postinstall",
    "prepare",
    "prepublish",
];

// ---------------------------------------------------------------------------
// Unsafe local path patterns
// ---------------------------------------------------------------------------

fn is_unsafe_local_path(path: &str) -> bool {
    let p = path.trim();
    // Path traversal
    if p.contains("..") {
        return true;
    }
    // Home path
    if p.starts_with("~/") || p.starts_with("~\\") {
        return true;
    }
    // Absolute sensitive paths
    let lower = p.to_lowercase();
    if lower.starts_with("/etc/")
        || lower.starts_with("/root/")
        || lower.starts_with("/home/")
        || lower.starts_with("/usr/")
        || lower.starts_with("/var/")
        || lower.starts_with("/tmp/")
        || lower.starts_with("/proc/")
        || lower.starts_with("/sys/")
        || lower.starts_with("/dev/")
        || lower.starts_with("c:\\")
        || lower.starts_with("\\\\")
    {
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Rejected output for raw-secret input
// ---------------------------------------------------------------------------

fn rejected_output(request: &InprocInvocation) -> Value {
    serde_json::json!({
        "kind": "project_intake_rejected",
        "redaction_state": "unsafe_blocked",
        "reason": "input contains raw-secret-like content; use secret_ref references instead",
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    })
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    let id = request.capability_id.as_str();
    if id.ends_with("/describe_intake_contract") {
        Some(describe_intake_contract(request))
    } else if id.ends_with("/inspect_external_project_ref") {
        Some(inspect_external_project_ref(request))
    } else if id.ends_with("/detect_project_stack_from_metadata") {
        Some(detect_project_stack_from_metadata(request))
    } else if id.ends_with("/draft_workspace_plan") {
        Some(draft_workspace_plan(request))
    } else if id.ends_with("/draft_security_risk_summary") {
        Some(draft_security_risk_summary(request))
    } else if id.ends_with("/list_candidate_entrypoints") {
        Some(list_candidate_entrypoints(request))
    } else if id.ends_with("/draft_adapter_plan") {
        Some(draft_adapter_plan(request))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Capability implementations
// ---------------------------------------------------------------------------

fn describe_intake_contract(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "project_intake_contract",
        "package_id": request.provider_package_id,
        "package_kind": "ordinary",
        "capabilities": [
            {"id": "official/project-intake-lab/describe_intake_contract", "purpose": "describe the project intake lab contract"},
            {"id": "official/project-intake-lab/inspect_external_project_ref", "purpose": "classify an external project source ref without clone/install/run"},
            {"id": "official/project-intake-lab/detect_project_stack_from_metadata", "purpose": "detect project stack from metadata files without filesystem access"},
            {"id": "official/project-intake-lab/draft_workspace_plan", "purpose": "draft a plan-only workspace plan, no direct workspace creation"},
            {"id": "official/project-intake-lab/draft_security_risk_summary", "purpose": "draft security risk summary from metadata, no filesystem scan"},
            {"id": "official/project-intake-lab/list_candidate_entrypoints", "purpose": "list candidate entrypoints with risk annotations, no execution"},
            {"id": "official/project-intake-lab/draft_adapter_plan", "purpose": "draft plan-only adapter plan, no direct adapter creation"},
        ],
        "surfaces": {
            "forge_panel": "official/project-intake-lab/forge-panel",
            "assistant_action": "official/project-intake-lab/assistant-action",
            "home_card": "official/project-intake-lab/home-card",
        },
        "source_kinds": SOURCE_KINDS,
        "stack_kinds": STACK_KINDS,
        "metadata_kinds": METADATA_KINDS,
        "npm_lifecycle_scripts": NPM_LIFECYCLE_SCRIPTS,
        "output_shapes": {
            "intake_contract": ["package_id", "package_kind", "capabilities", "surfaces", "source_kinds", "stack_kinds", "metadata_kinds", "npm_lifecycle_scripts"],
            "project_ref_inspection": ["source_kind", "source_ref", "classification_confidence", "path_safety", "unsafe_path_reason", "warnings"],
            "stack_detection": ["detected_stack", "metadata_signals", "confidence", "npm_lifecycle_risks"],
            "workspace_plan": ["plan_only", "requires_user_approval", "source_kind", "source_ref", "proposed_steps", "risk_notes"],
            "security_risk_summary": ["risk_level", "risk_factors", "npm_lifecycle_risks", "path_safety", "raw_secret_detected", "recommendations"],
            "candidate_entrypoints": ["entrypoints", "entrypoints[].label", "entrypoints[].command", "entrypoints[].requires_approval", "entrypoints[].executes_code"],
            "adapter_plan": ["plan_only", "requires_user_approval", "source_kind", "proposed_capabilities", "proposed_entry", "risk_notes"],
        },
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn inspect_external_project_ref(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }

    let source_ref = request
        .input
        .get("source_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    let source_kind = request
        .input
        .get("source_kind")
        .and_then(Value::as_str)
        .filter(|k| SOURCE_KINDS.contains(k))
        .unwrap_or_else(|| classify_source_kind(source_ref));

    let (path_safety, unsafe_path_reason) = if source_kind == "local" {
        if is_unsafe_local_path(source_ref) {
            (
                "rejected",
                Some("unsafe local path: path traversal, home path, or absolute sensitive path"),
            )
        } else if source_ref.is_empty() {
            ("unknown", None)
        } else {
            ("appears_safe", None)
        }
    } else {
        ("not_applicable", None)
    };

    let mut warnings: Vec<Value> = Vec::new();
    if source_kind == "unknown" {
        warnings.push(serde_json::json!({
            "kind": "unknown_source",
            "message": "source ref could not be classified; manual review recommended"
        }));
    }
    if path_safety == "rejected" {
        warnings.push(serde_json::json!({
            "kind": "unsafe_path",
            "message": unsafe_path_reason.unwrap_or("unsafe local path")
        }));
    }

    let classification_confidence = match source_kind {
        "git" | "npm" | "archive" => "high",
        "local" => "medium",
        _ => "low",
    };

    Ok(serde_json::json!({
        "kind": "project_ref_inspection",
        "source_kind": source_kind,
        "source_ref": if source_ref.is_empty() { Value::Null } else { serde_json::json!(source_ref) },
        "classification_confidence": classification_confidence,
        "path_safety": path_safety,
        "unsafe_path_reason": if path_safety == "rejected" { serde_json::json!(unsafe_path_reason) } else { Value::Null },
        "warnings": warnings,
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn detect_project_stack_from_metadata(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }

    let metadata = request
        .input
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut signals: Vec<Value> = Vec::new();
    let mut npm_lifecycle_risks: Vec<Value> = Vec::new();

    // Detect from metadata signals
    if metadata.contains_key("package_json") {
        signals
            .push(serde_json::json!({"metadata_kind": "package_json", "signal": "node_project"}));
        // Check for npm lifecycle scripts
        let pkg = metadata.get("package_json");
        if let Some(scripts) = pkg
            .and_then(Value::as_object)
            .and_then(|o| o.get("scripts"))
            .and_then(Value::as_object)
        {
            for &script_name in NPM_LIFECYCLE_SCRIPTS {
                if scripts.contains_key(script_name) {
                    npm_lifecycle_risks.push(serde_json::json!({
                        "script": script_name,
                        "risk": "executes_code",
                        "description": format!("npm `{}` script runs arbitrary code during install", script_name),
                        "requires_approval": true,
                        "executes_code": true,
                    }));
                }
            }
        }
    }
    if metadata.contains_key("cargo_toml") {
        signals.push(serde_json::json!({"metadata_kind": "cargo_toml", "signal": "rust_project"}));
    }
    if metadata.contains_key("pyproject") {
        signals.push(serde_json::json!({"metadata_kind": "pyproject", "signal": "python_project"}));
    }
    if metadata.contains_key("readme") {
        signals.push(serde_json::json!({"metadata_kind": "readme", "signal": "documentation"}));
    }
    if metadata.contains_key("files") {
        signals.push(serde_json::json!({"metadata_kind": "files", "signal": "file_listing"}));
    }

    // Determine stack
    let detected_stack = if signals.iter().any(|s| s["signal"] == "node_project") {
        "node"
    } else if signals.iter().any(|s| s["signal"] == "rust_project") {
        "rust"
    } else if signals.iter().any(|s| s["signal"] == "python_project") {
        "python"
    } else if !signals.is_empty() {
        "static"
    } else {
        "unknown"
    };

    let confidence = if !signals.is_empty() { "medium" } else { "low" };

    Ok(serde_json::json!({
        "kind": "project_stack_detection",
        "detected_stack": detected_stack,
        "metadata_signals": signals,
        "confidence": confidence,
        "npm_lifecycle_risks": npm_lifecycle_risks,
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn draft_workspace_plan(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }

    let source_ref = request
        .input
        .get("source_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    let source_kind = request
        .input
        .get("source_kind")
        .and_then(Value::as_str)
        .filter(|k| SOURCE_KINDS.contains(k))
        .unwrap_or_else(|| classify_source_kind(source_ref));

    // Check local path safety
    if source_kind == "local" && is_unsafe_local_path(source_ref) {
        return Ok(serde_json::json!({
            "kind": "project_intake_rejected",
            "redaction_state": "unsafe_blocked",
            "reason": "unsafe local path in workspace plan: path traversal, home path, or absolute sensitive path",
            "inference_performed": false,
            "network_performed": false,
            "execution_performed": false,
            "filesystem_performed": false,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let mut proposed_steps: Vec<Value> = Vec::new();
    let mut risk_notes: Vec<Value> = Vec::new();

    match source_kind {
        "git" => {
            proposed_steps.push(serde_json::json!({"step": "clone", "requires_approval": true, "executes_code": false, "network_required": true}));
            proposed_steps.push(serde_json::json!({"step": "read_metadata", "requires_approval": false, "executes_code": false, "network_required": false}));
            risk_notes.push(serde_json::json!({"kind": "network_required", "note": "git clone requires outbound network access; must be policy-gated"}));
        }
        "npm" => {
            proposed_steps.push(serde_json::json!({"step": "fetch_package", "requires_approval": true, "executes_code": false, "network_required": true}));
            proposed_steps.push(serde_json::json!({"step": "read_metadata", "requires_approval": false, "executes_code": false, "network_required": false}));
            risk_notes.push(serde_json::json!({"kind": "network_required", "note": "npm fetch requires outbound network access; must be policy-gated"}));
        }
        "local" => {
            proposed_steps.push(serde_json::json!({"step": "read_metadata", "requires_approval": false, "executes_code": false, "network_required": false}));
        }
        "archive" => {
            proposed_steps.push(serde_json::json!({"step": "extract", "requires_approval": true, "executes_code": false, "network_required": false}));
            proposed_steps.push(serde_json::json!({"step": "read_metadata", "requires_approval": false, "executes_code": false, "network_required": false}));
            risk_notes.push(serde_json::json!({"kind": "archive_risk", "note": "archive extraction must be path-bounded to prevent zip-slip"}));
        }
        _ => {
            proposed_steps.push(serde_json::json!({"step": "read_metadata", "requires_approval": false, "executes_code": false, "network_required": false}));
            risk_notes.push(serde_json::json!({"kind": "unknown_source", "note": "source kind unknown; manual review required"}));
        }
    }

    Ok(serde_json::json!({
        "kind": "project_workspace_plan",
        "plan_only": true,
        "requires_user_approval": true,
        "source_kind": source_kind,
        "source_ref": if source_ref.is_empty() { Value::Null } else { serde_json::json!(source_ref) },
        "proposed_steps": proposed_steps,
        "risk_notes": risk_notes,
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn draft_security_risk_summary(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }

    let source_ref = request
        .input
        .get("source_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    let source_kind = request
        .input
        .get("source_kind")
        .and_then(Value::as_str)
        .filter(|k| SOURCE_KINDS.contains(k))
        .unwrap_or_else(|| classify_source_kind(source_ref));

    let metadata = request
        .input
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut risk_factors: Vec<Value> = Vec::new();
    let mut npm_lifecycle_risks: Vec<Value> = Vec::new();
    let mut recommendations: Vec<Value> = Vec::new();
    let mut path_safety = "not_applicable";

    // Path safety for local
    if source_kind == "local" {
        if is_unsafe_local_path(source_ref) {
            path_safety = "rejected";
            risk_factors.push(serde_json::json!({"kind": "unsafe_path", "severity": "high", "detail": "path traversal, home path, or absolute sensitive path"}));
        } else if source_ref.is_empty() {
            path_safety = "unknown";
        } else {
            path_safety = "appears_safe";
        }
    }

    // npm lifecycle risks
    if let Some(scripts) = metadata
        .get("package_json")
        .and_then(Value::as_object)
        .and_then(|o| o.get("scripts"))
        .and_then(Value::as_object)
    {
        for &script_name in NPM_LIFECYCLE_SCRIPTS {
            if scripts.contains_key(script_name) {
                npm_lifecycle_risks.push(serde_json::json!({
                    "script": script_name,
                    "risk": "executes_code",
                    "description": format!("npm `{}` script runs arbitrary code during install", script_name),
                    "requires_approval": true,
                    "executes_code": true,
                }));
            }
        }
        if !npm_lifecycle_risks.is_empty() {
            risk_factors.push(serde_json::json!({
                "kind": "npm_lifecycle_scripts",
                "severity": "high",
                "detail": format!("{} npm lifecycle script(s) that execute code detected", npm_lifecycle_risks.len())
            }));
            recommendations.push(serde_json::json!({
                "kind": "skip_lifecycle_scripts",
                "suggestion": "use --ignore-scripts or equivalent flag to prevent automatic code execution"
            }));
        }
    }

    // Source-specific risks
    match source_kind {
        "git" => {
            risk_factors.push(serde_json::json!({"kind": "network_access", "severity": "medium", "detail": "git clone requires network access"}));
            recommendations.push(serde_json::json!({"kind": "network_policy", "suggestion": "require explicit network approval and audit for git operations"}));
        }
        "npm" => {
            risk_factors.push(serde_json::json!({"kind": "network_access", "severity": "medium", "detail": "npm fetch requires network access"}));
            risk_factors.push(serde_json::json!({"kind": "supply_chain", "severity": "high", "detail": "npm packages may contain malicious lifecycle scripts"}));
            recommendations.push(serde_json::json!({"kind": "audit_registry", "suggestion": "verify package integrity and audit npm registry source"}));
        }
        "archive" => {
            risk_factors.push(serde_json::json!({"kind": "archive_path_traversal", "severity": "medium", "detail": "archives may contain path-traversal entries"}));
        }
        "local" => {}
        _ => {
            risk_factors.push(serde_json::json!({"kind": "unknown_source", "severity": "medium", "detail": "source kind unknown; risk cannot be fully assessed"}));
        }
    }

    let raw_secret_detected = safety::contains_raw_secret(&request.input);
    if raw_secret_detected {
        // Already handled above, but defensive check
        risk_factors.push(serde_json::json!({"kind": "raw_secret", "severity": "critical", "detail": "raw secret-like content detected in input"}));
    }

    let risk_level =
        if raw_secret_detected || path_safety == "rejected" || !npm_lifecycle_risks.is_empty() {
            "high"
        } else if risk_factors.is_empty() {
            "low"
        } else {
            "medium"
        };

    Ok(serde_json::json!({
        "kind": "project_security_risk_summary",
        "risk_level": risk_level,
        "risk_factors": risk_factors,
        "npm_lifecycle_risks": npm_lifecycle_risks,
        "path_safety": path_safety,
        "raw_secret_detected": raw_secret_detected,
        "recommendations": recommendations,
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn list_candidate_entrypoints(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }

    let metadata = request
        .input
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut entrypoints: Vec<Value> = Vec::new();

    // Node project entrypoints
    if let Some(pkg) = metadata.get("package_json").and_then(Value::as_object) {
        if pkg.contains_key("bin") {
            entrypoints.push(serde_json::json!({
                "label": "bin (package.json)",
                "command": "bin",
                "source": "package_json",
                "requires_approval": true,
                "executes_code": true,
            }));
        }
        if let Some(main) = pkg.get("main") {
            entrypoints.push(serde_json::json!({
                "label": format!("main ({})", main),
                "command": format!("node {}", main),
                "source": "package_json",
                "requires_approval": true,
                "executes_code": true,
            }));
        }
        if let Some(scripts) = pkg.get("scripts").and_then(Value::as_object) {
            if scripts.contains_key("start") {
                entrypoints.push(serde_json::json!({
                    "label": "npm start",
                    "command": "npm start",
                    "source": "package_json.scripts",
                    "requires_approval": true,
                    "executes_code": true,
                }));
            }
            if scripts.contains_key("test") {
                entrypoints.push(serde_json::json!({
                    "label": "npm test",
                    "command": "npm test",
                    "source": "package_json.scripts",
                    "requires_approval": true,
                    "executes_code": true,
                }));
            }
            if scripts.contains_key("dev") {
                entrypoints.push(serde_json::json!({
                    "label": "npm run dev",
                    "command": "npm run dev",
                    "source": "package_json.scripts",
                    "requires_approval": true,
                    "executes_code": true,
                }));
            }
        }
    }

    // Rust project entrypoints
    if metadata.contains_key("cargo_toml") {
        entrypoints.push(serde_json::json!({
            "label": "cargo build",
            "command": "cargo build",
            "source": "cargo_toml",
            "requires_approval": true,
            "executes_code": true,
        }));
        entrypoints.push(serde_json::json!({
            "label": "cargo test",
            "command": "cargo test",
            "source": "cargo_toml",
            "requires_approval": true,
            "executes_code": true,
        }));
    }

    // Python project entrypoints
    if metadata.contains_key("pyproject") {
        entrypoints.push(serde_json::json!({
            "label": "pip install",
            "command": "pip install",
            "source": "pyproject",
            "requires_approval": true,
            "executes_code": true,
        }));
        if let Some(pyproj) = metadata.get("pyproject").and_then(Value::as_object) {
            if let Some(scripts) = pyproj.get("scripts").and_then(Value::as_object) {
                for name in scripts.keys() {
                    entrypoints.push(serde_json::json!({
                        "label": name,
                        "command": name,
                        "source": "pyproject.scripts",
                        "requires_approval": true,
                        "executes_code": true,
                    }));
                }
            }
        }
    }

    // README suggestions
    if metadata.contains_key("readme") {
        entrypoints.push(serde_json::json!({
            "label": "README review",
            "command": "read",
            "source": "readme",
            "requires_approval": false,
            "executes_code": false,
        }));
    }

    Ok(serde_json::json!({
        "kind": "project_candidate_entrypoints",
        "entrypoints": entrypoints,
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn draft_adapter_plan(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }

    let source_ref = request
        .input
        .get("source_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    let source_kind = request
        .input
        .get("source_kind")
        .and_then(Value::as_str)
        .filter(|k| SOURCE_KINDS.contains(k))
        .unwrap_or_else(|| classify_source_kind(source_ref));

    // Check local path safety
    if source_kind == "local" && is_unsafe_local_path(source_ref) {
        return Ok(serde_json::json!({
            "kind": "project_intake_rejected",
            "redaction_state": "unsafe_blocked",
            "reason": "unsafe local path in adapter plan: path traversal, home path, or absolute sensitive path",
            "inference_performed": false,
            "network_performed": false,
            "execution_performed": false,
            "filesystem_performed": false,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let mut proposed_capabilities: Vec<Value> = Vec::new();
    let mut risk_notes: Vec<Value> = Vec::new();

    match source_kind {
        "git" | "npm" | "local" | "archive" => {
            proposed_capabilities.push(serde_json::json!({
                "capability_kind": "invoke",
                "purpose": "run a project command through the adapter",
                "requires_approval": true,
            }));
            proposed_capabilities.push(serde_json::json!({
                "capability_kind": "inspect",
                "purpose": "read project metadata through the adapter",
                "requires_approval": false,
            }));
            if source_kind == "git" || source_kind == "npm" {
                risk_notes.push(serde_json::json!({"kind": "network_required", "note": "adapter may need network access for git clone or npm fetch"}));
            }
        }
        _ => {
            risk_notes.push(serde_json::json!({"kind": "unknown_source", "note": "source kind unknown; adapter plan may be incomplete"}));
        }
    }

    Ok(serde_json::json!({
        "kind": "project_adapter_plan",
        "plan_only": true,
        "requires_user_approval": true,
        "source_kind": source_kind,
        "source_ref": if source_ref.is_empty() { Value::Null } else { serde_json::json!(source_ref) },
        "proposed_capabilities": proposed_capabilities,
        "proposed_entry": {
            "kind": "subprocess",
            "requires_approval": true,
        },
        "risk_notes": risk_notes,
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

// ---------------------------------------------------------------------------
// Source kind classification
// ---------------------------------------------------------------------------

fn classify_source_kind(source_ref: &str) -> &'static str {
    if source_ref.starts_with("https://")
        || source_ref.starts_with("git://")
        || source_ref.starts_with("git@")
        || source_ref.ends_with(".git")
    {
        "git"
    } else if source_ref.starts_with("npm:")
        || source_ref.starts_with("@") && source_ref.contains('/') && !source_ref.starts_with("@/")
    {
        "npm"
    } else if source_ref.ends_with(".tar.gz")
        || source_ref.ends_with(".zip")
        || source_ref.ends_with(".tgz")
        || source_ref.ends_with(".tar.xz")
    {
        "archive"
    } else if source_ref.starts_with("./")
        || source_ref.starts_with("../")
        || source_ref.starts_with("/")
        || source_ref.starts_with("~/")
        || (!source_ref.contains(':') && !source_ref.contains('@') && !source_ref.is_empty())
    {
        "local"
    } else {
        "unknown"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_request(cap: &str, input: Value) -> InprocInvocation {
        InprocInvocation {
            capability_id: cap.to_string(),
            provider_package_id: PACKAGE_ID.to_string(),
            input,
        }
    }

    #[test]
    fn try_handle_matches_package_id() {
        let req = make_request(
            "official/project-intake-lab/describe_intake_contract",
            json!({}),
        );
        assert!(try_handle(&req).is_some());
    }

    #[test]
    fn try_handle_rejects_wrong_package() {
        let req = InprocInvocation {
            capability_id: "official/project-intake-lab/describe_intake_contract".to_string(),
            provider_package_id: "official/other".to_string(),
            input: json!({}),
        };
        assert!(try_handle(&req).is_none());
    }

    #[test]
    fn describe_contract_has_all_surfaces() {
        let req = make_request(
            "official/project-intake-lab/describe_intake_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        let surfaces = result["surfaces"].as_object().unwrap();
        assert!(surfaces.contains_key("forge_panel"));
        assert!(surfaces.contains_key("assistant_action"));
        assert!(surfaces.contains_key("home_card"));
    }

    #[test]
    fn describe_contract_lists_7_capabilities() {
        let req = make_request(
            "official/project-intake-lab/describe_intake_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(
            result["capabilities"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            7,
            "must list 7 capabilities"
        );
    }

    #[test]
    fn inspect_classifies_git() {
        let req = make_request(
            "official/project-intake-lab/inspect_external_project_ref",
            json!({"source_ref": "https://github.com/example/project.git"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["source_kind"], json!("git"));
    }

    #[test]
    fn inspect_classifies_npm() {
        let req = make_request(
            "official/project-intake-lab/inspect_external_project_ref",
            json!({"source_ref": "npm:lodash"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["source_kind"], json!("npm"));
    }

    #[test]
    fn inspect_classifies_local() {
        let req = make_request(
            "official/project-intake-lab/inspect_external_project_ref",
            json!({"source_ref": "./my-project"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["source_kind"], json!("local"));
        assert_eq!(result["path_safety"], json!("appears_safe"));
    }

    #[test]
    fn inspect_rejects_unsafe_path_traversal() {
        let req = make_request(
            "official/project-intake-lab/inspect_external_project_ref",
            json!({"source_ref": "../../etc/passwd"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["path_safety"], json!("rejected"));
    }

    #[test]
    fn inspect_rejects_home_path() {
        let req = make_request(
            "official/project-intake-lab/inspect_external_project_ref",
            json!({"source_ref": "~/secret-project"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["path_safety"], json!("rejected"));
    }

    #[test]
    fn inspect_rejects_absolute_sensitive_path() {
        let req = make_request(
            "official/project-intake-lab/inspect_external_project_ref",
            json!({"source_ref": "/etc/shadow"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["path_safety"], json!("rejected"));
    }

    #[test]
    fn detect_stack_node() {
        let req = make_request(
            "official/project-intake-lab/detect_project_stack_from_metadata",
            json!({"metadata": {"package_json": {"name": "test", "scripts": {"start": "node index.js"}}}}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["detected_stack"], json!("node"));
        let risks = result["npm_lifecycle_risks"].as_array().unwrap();
        // "start" is not a lifecycle script, so no npm_lifecycle_risks
        assert_eq!(risks.len(), 0);
    }

    #[test]
    fn detect_stack_node_with_lifecycle_scripts() {
        let req = make_request(
            "official/project-intake-lab/detect_project_stack_from_metadata",
            json!({"metadata": {"package_json": {"name": "test", "scripts": {"preinstall": "echo hi", "postinstall": "echo bye", "start": "node index.js"}}}}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["detected_stack"], json!("node"));
        let risks = result["npm_lifecycle_risks"].as_array().unwrap();
        assert_eq!(risks.len(), 2); // preinstall + postinstall
        for risk in risks {
            assert_eq!(risk["executes_code"], json!(true));
            assert_eq!(risk["requires_approval"], json!(true));
        }
    }

    #[test]
    fn detect_stack_rust() {
        let req = make_request(
            "official/project-intake-lab/detect_project_stack_from_metadata",
            json!({"metadata": {"cargo_toml": {"name": "test"}}}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["detected_stack"], json!("rust"));
    }

    #[test]
    fn detect_stack_python() {
        let req = make_request(
            "official/project-intake-lab/detect_project_stack_from_metadata",
            json!({"metadata": {"pyproject": {"name": "test"}}}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["detected_stack"], json!("python"));
    }

    #[test]
    fn detect_stack_unknown() {
        let req = make_request(
            "official/project-intake-lab/detect_project_stack_from_metadata",
            json!({"metadata": {}}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["detected_stack"], json!("unknown"));
    }

    #[test]
    fn workspace_plan_is_plan_only() {
        let req = make_request(
            "official/project-intake-lab/draft_workspace_plan",
            json!({"source_ref": "https://github.com/example/project.git", "source_kind": "git"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("project_workspace_plan"));
        assert_eq!(result["plan_only"], json!(true));
        assert_eq!(result["requires_user_approval"], json!(true));
    }

    #[test]
    fn workspace_plan_rejects_unsafe_local_path() {
        let req = make_request(
            "official/project-intake-lab/draft_workspace_plan",
            json!({"source_ref": "~/secret-project", "source_kind": "local"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("project_intake_rejected"));
        assert_eq!(result["redaction_state"], json!("unsafe_blocked"));
    }

    #[test]
    fn adapter_plan_is_plan_only() {
        let req = make_request(
            "official/project-intake-lab/draft_adapter_plan",
            json!({"source_ref": "./my-project", "source_kind": "local"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("project_adapter_plan"));
        assert_eq!(result["plan_only"], json!(true));
        assert_eq!(result["requires_user_approval"], json!(true));
    }

    #[test]
    fn raw_secret_blocked() {
        let req = make_request(
            "official/project-intake-lab/inspect_external_project_ref",
            json!({"source_ref": "test", "api_key": "RawSecretExample1234567890abcdefABCDEF123456"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("project_intake_rejected"));
        assert_eq!(result["redaction_state"], json!("unsafe_blocked"));
    }

    #[test]
    fn no_forbidden_namespace_in_contract() {
        let req = make_request(
            "official/project-intake-lab/describe_intake_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        let output_str = serde_json::to_string(&result).unwrap();
        for token in &[
            "kernel.project.",
            "kernel.workspace.",
            "kernel.git.",
            "kernel.npm.",
            "kernel.deploy.",
            "kernel.ide.",
        ] {
            assert!(!output_str.contains(token), "must not contain {}", token);
        }
    }

    #[test]
    fn candidate_entrypoints_require_approval() {
        let req = make_request(
            "official/project-intake-lab/list_candidate_entrypoints",
            json!({"metadata": {"package_json": {"name": "test", "main": "index.js", "scripts": {"start": "node index.js"}}}}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        let entrypoints = result["entrypoints"].as_array().unwrap();
        assert!(!entrypoints.is_empty());
        for ep in entrypoints {
            if ep["executes_code"] == json!(true) {
                assert_eq!(
                    ep["requires_approval"],
                    json!(true),
                    "executes_code entrypoint must require approval"
                );
            }
        }
    }

    #[test]
    fn no_execution_performed() {
        let caps = [
            "describe_intake_contract",
            "inspect_external_project_ref",
            "detect_project_stack_from_metadata",
            "draft_workspace_plan",
            "draft_security_risk_summary",
            "list_candidate_entrypoints",
            "draft_adapter_plan",
        ];
        for cap in &caps {
            let req = make_request(&format!("official/project-intake-lab/{}", cap), json!({}));
            let result = try_handle(&req).unwrap().unwrap();
            assert_eq!(
                result["execution_performed"],
                json!(false),
                "{} must have execution_performed=false",
                cap
            );
            assert_eq!(
                result["network_performed"],
                json!(false),
                "{} must have network_performed=false",
                cap
            );
            assert_eq!(
                result["inference_performed"],
                json!(false),
                "{} must have inference_performed=false",
                cap
            );
            assert_eq!(
                result["filesystem_performed"],
                json!(false),
                "{} must have filesystem_performed=false",
                cap
            );
        }
    }

    #[test]
    fn is_unsafe_local_path_tests() {
        // Path traversal
        assert!(is_unsafe_local_path("../../etc/passwd"));
        assert!(is_unsafe_local_path("foo/../bar/.."));
        // Home path
        assert!(is_unsafe_local_path("~/project"));
        // Absolute sensitive
        assert!(is_unsafe_local_path("/etc/shadow"));
        assert!(is_unsafe_local_path("/root/.ssh"));
        assert!(is_unsafe_local_path("/home/user/.ssh"));
        // Safe paths
        assert!(!is_unsafe_local_path("./my-project"));
        assert!(!is_unsafe_local_path("my-project"));
    }

    #[test]
    fn classify_source_kind_tests() {
        assert_eq!(
            classify_source_kind("https://github.com/example/project.git"),
            "git"
        );
        assert_eq!(
            classify_source_kind("git@github.com:example/project.git"),
            "git"
        );
        assert_eq!(classify_source_kind("npm:lodash"), "npm");
        assert_eq!(classify_source_kind("@scope/package"), "npm");
        assert_eq!(classify_source_kind("./my-project"), "local");
        assert_eq!(classify_source_kind("project.tar.gz"), "archive");
        assert_eq!(classify_source_kind("unknown-thing"), "local");
        assert_eq!(classify_source_kind(""), "unknown");
    }
}
