use serde_json::{json, Value};
use ygg_core::*;
use ygg_runtime::*;

use super::defs::*;
use super::write::filename;
use super::{BASE, SCHEMA};

pub(crate) fn event_schema(kind: &str, payload: Value) -> Value {
    json!({
        "$schema": SCHEMA,
        "$id": format!("{BASE}/events/{}.schema.json", filename(kind)),
        "title": kind,
        "description": format!("Payload schema for event kind {kind}."),
        "type": "object",
        "properties": { "kind": { "const": kind }, "payload": { "$ref": "#/$defs/Payload" } },
        "$defs": { "Payload": payload }
    })
}

pub(crate) fn event_schemas() -> Vec<(&'static str, Value)> {
    vec![
        (EVENT_SESSION_OPENED, json!({"type":"object"})),
        (EVENT_SESSION_CLOSED, json!({"type":"object"})),
        (EVENT_SESSION_FORKED, schema_value::<BranchRecord>()),
        (
            EVENT_PACKAGE_LOADED,
            schema_value::<PackageLifecyclePayload>(),
        ),
        (
            EVENT_PACKAGE_LOADING,
            schema_value::<PackageLifecyclePayload>(),
        ),
        (
            EVENT_PACKAGE_STARTING,
            schema_value::<PackageLifecyclePayload>(),
        ),
        (
            EVENT_PACKAGE_READY,
            schema_value::<PackageLifecyclePayload>(),
        ),
        (
            EVENT_PACKAGE_STOPPING,
            schema_value::<PackageLifecyclePayload>(),
        ),
        (
            EVENT_PACKAGE_STOPPED,
            schema_value::<PackageLifecyclePayload>(),
        ),
        (
            EVENT_PACKAGE_UNLOADED,
            schema_value::<PackageLifecyclePayload>(),
        ),
        (
            EVENT_PACKAGE_DEGRADED,
            schema_value::<PackageLifecyclePayload>(),
        ),
        (EVENT_PACKAGE_LOG, schema_value::<SubprocessLogLine>()),
        (
            PROJECT_INSTALLED,
            schema_value::<ProjectLifecyclePayloadSchema>(),
        ),
        (
            PROJECT_STARTED,
            schema_value::<ProjectLifecyclePayloadSchema>(),
        ),
        (
            PROJECT_STOPPED,
            schema_value::<ProjectLifecyclePayloadSchema>(),
        ),
        (
            PROJECT_UNINSTALLED,
            schema_value::<ProjectLifecyclePayloadSchema>(),
        ),
        (EVENT_ASSET_PUT, schema_value::<AssetRecord>()),
        (
            EVENT_PROJECTION_UPDATED,
            schema_value::<ProjectionDefinition>(),
        ),
        (EVENT_PROPOSAL_CREATED, schema_value::<ProposalRecord>()),
        (EVENT_PROPOSAL_APPROVED, schema_value::<ProposalRecord>()),
        (EVENT_PROPOSAL_REJECTED, schema_value::<ProposalRecord>()),
        (EVENT_PROPOSAL_APPLIED, schema_value::<ProposalRecord>()),
        (EVENT_PROPOSAL_FAILED, schema_value::<ProposalRecord>()),
        (EVENT_CAPABILITY_INVOKED, json!({"type":"object"})),
        (
            EVENT_CAPABILITY_COMPLETED,
            schema_value::<CapabilityInvocationResult>(),
        ),
        (EVENT_CAPABILITY_FAILED, json!({"type":"object"})),
        (
            EVENT_PERMISSION_DENIED,
            json!({"type":"object","properties":{"package_id":{"type":"string"},"operation":{"type":"string"}}}),
        ),
        (
            EVENT_PERMISSION_GRANTED,
            schema_value::<PermissionGrantRecord>(),
        ),
        (
            EVENT_PERMISSION_REVOKED,
            schema_value::<PermissionGrantRecord>(),
        ),
        (EVENT_ERROR, schema_value::<ErrorShape>()),
        (
            EVENT_OUTBOUND_REQUEST,
            schema_value::<OutboundAuditRecord>(),
        ),
        (EVENT_OUTBOUND_DENIED, schema_value::<OutboundAuditRecord>()),
        (EVENT_OUTBOUND_EXECUTE_COMPLETED, json!({"type":"object"})),
        (
            EVENT_OUTBOUND_STREAM_COMPLETED,
            schema_value::<OutboundStreamSummary>(),
        ),
        (EVENT_STREAM_STARTED, json!({"type":"object"})),
        (EVENT_STREAM_CHUNK, schema_value::<StreamFrameEnvelope>()),
        (EVENT_STREAM_PROGRESS, schema_value::<StreamFrameEnvelope>()),
        (EVENT_STREAM_ENDED, schema_value::<StreamFrameEnvelope>()),
        (EVENT_STREAM_ERROR, schema_value::<StreamFrameEnvelope>()),
        (
            EVENT_STREAM_CANCELLED,
            schema_value::<StreamFrameEnvelope>(),
        ),
        (EVENT_STREAM_TIMEOUT, schema_value::<StreamFrameEnvelope>()),
        (EVENT_OUTBOUND_WEBSOCKET_OPENED, json!({"type":"object"})),
        (EVENT_OUTBOUND_WEBSOCKET_FRAME, json!({"type":"object"})),
        (EVENT_OUTBOUND_WEBSOCKET_ERROR, json!({"type":"object"})),
        (EVENT_OUTBOUND_WEBSOCKET_COMPLETED, json!({"type":"object"})),
        (EVENT_EXEC_REQUEST, json!({"type":"object"})),
        (EVENT_EXEC_DENIED, json!({"type":"object"})),
        (EVENT_EXEC_STARTED, json!({"type":"object"})),
        (EVENT_EXEC_STOPPED, json!({"type":"object"})),
        (EVENT_EXEC_COMPLETED, json!({"type":"object"})),
        (EVENT_EXEC_FAILED, json!({"type":"object"})),
        (EVENT_PORT_LEASED, json!({"type":"object"})),
        (EVENT_PORT_RELEASED, json!({"type":"object"})),
        (EVENT_PORT_DENIED, json!({"type":"object"})),
        (EVENT_PROXY_REGISTERED, json!({"type":"object"})),
        (EVENT_PROXY_UNREGISTERED, json!({"type":"object"})),
        (EVENT_PROXY_DENIED, json!({"type":"object"})),
        (
            EVENT_DEPLOYMENT_RECONCILED,
            schema_value::<DeploymentReconcileSummary>(),
        ),
        (
            EVENT_DEPLOYMENT_HEALTH,
            schema_value::<DeploymentHealthEventPayload>(),
        ),
    ]
}
