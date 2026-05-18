//! Handler for `thirdparty/agent-runtime` capabilities.
//!
//! Demonstrates that a third-party agent runtime can produce the same
//! deterministic, no-network, no-inference, proposal-gated output shapes
//! as the official `official/pi-agent-runtime-lab` without any kernel
//! privilege or special routing.

use serde_json::Value;

use super::InprocInvocation;

const PACKAGE_ID: &str = "thirdparty/agent-runtime";

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    let id = request.capability_id.as_str();
    if id.ends_with("/run") {
        Some(run(request))
    } else if id.ends_with("/explain_run") {
        Some(explain_run(request))
    } else if id.ends_with("/draft_proposal") {
        Some(draft_proposal(request))
    } else if id.ends_with("/summarize_trace") {
        Some(summarize_trace(request))
    } else if id.ends_with("/echo") {
        Some(echo(request))
    } else {
        None
    }
}

fn run(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "thirdparty_agent_run_plan",
        "inference_performed": false,
        "network_performed": false,
        "trace_events": [
            {
                "event_type": "capability_invoked",
                "capability_id": request.capability_id,
                "timestamp": 0,
                "payload": {"step": "plan_only", "status": "deterministic"}
            }
        ],
        "stream_frames": [
            {"frame_type": "start", "sequence": 0},
            {"frame_type": "chunk", "sequence": 1, "content": "community deterministic plan step"},
            {"frame_type": "progress", "sequence": 2, "percent": 100},
            {"frame_type": "end", "sequence": 3}
        ],
        "proposal_draft": {
            "kind": "thirdparty_agent_proposal_draft",
            "requires_user_approval": true,
            "recommended_operation": "kernel.session.fork",
            "plan_summary": "deterministic no-inference agent run plan from community runtime"
        },
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn explain_run(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "thirdparty_agent_run_explanation",
        "summary": "Community agent run plan explanation: no inference or network was performed.",
        "inference_performed": false,
        "network_performed": false,
        "trace_event_count": request.input.get("trace_events").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0),
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn draft_proposal(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "thirdparty_agent_proposal",
        "requires_user_approval": true,
        "recommended_operation": "kernel.session.fork",
        "proposal": request.input,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn summarize_trace(request: &InprocInvocation) -> anyhow::Result<Value> {
    let event_count = request.input.get("trace_events").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
    Ok(serde_json::json!({
        "kind": "thirdparty_agent_trace_summary",
        "event_count": event_count,
        "inference_performed": false,
        "network_performed": false,
        "summary": format!("Community trace summary: {event_count} events, no inference, no network"),
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn echo(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "thirdparty_agent_echo",
        "input": request.input,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}
