#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use schemars::{schema_for, JsonSchema};
use serde_json::{json, Value};
use ygg_core::*;
use ygg_runtime::*;

const SCHEMA: &str = "https://json-schema.org/draft/2020-12/schema";
const BASE: &str = "https://yggdrasil.dev/spec/v1";

#[derive(JsonSchema)]
struct EmptyParams {}

#[derive(JsonSchema)]
struct ErrorShape {
    code: String,
    message: String,
    details: Value,
}

#[derive(JsonSchema)]
struct SessionCloseParams {
    session_id: String,
}
#[derive(JsonSchema)]
struct SessionGetParams {
    session_id: String,
}
#[derive(JsonSchema)]
struct SessionForkParams {
    parent_session_id: String,
    forked_from_sequence: u64,
    #[schemars(schema_with = "json_value_schema")]
    metadata: Value,
}
#[derive(JsonSchema)]
struct SessionBranchListParams {
    session_id: String,
}
#[derive(JsonSchema)]
struct PackageIdParams {
    package_id: String,
}
#[derive(JsonSchema)]
struct ProjectIdParams {
    project_id: ygg_core::project::ProjectId,
}
#[derive(JsonSchema)]
struct ProjectListParams {
    filter_state: Option<ygg_core::project::ProjectState>,
}
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum StorageMeasurementStateSchema {
    Measured,
    Unknown,
}
#[derive(JsonSchema)]
struct ProjectStorageSummarySchema {
    data_bytes: Option<u64>,
    cache_bytes: Option<u64>,
    bundle_bytes: Option<u64>,
    log_bytes: Option<u64>,
    total_bytes: Option<u64>,
    measured_at: Option<chrono::DateTime<chrono::Utc>>,
    measurement_state: StorageMeasurementStateSchema,
}
#[derive(JsonSchema)]
struct ProjectListItemSchema {
    id: ygg_core::project::ProjectId,
    title: String,
    description: String,
    #[serde(rename = "type")]
    project_type: ygg_core::project::ProjectType,
    state: ygg_core::project::ProjectState,
    icon: Option<String>,
    entry_surface_id: Option<String>,
    storage_summary: Option<ProjectStorageSummarySchema>,
    #[schemars(schema_with = "string_schema")]
    running_session_id: Option<String>,
}
#[derive(JsonSchema)]
struct ProjectListResultSchema {
    projects: Vec<ProjectListItemSchema>,
}
#[derive(JsonSchema)]
struct ProjectStartResult {
    project_id: ygg_core::project::ProjectId,
    previous_state: ygg_core::project::ProjectState,
    new_state: ygg_core::project::ProjectState,
    session_id: String,
    already_running: bool,
}
#[derive(JsonSchema)]
struct ProjectStopResult {
    project_id: ygg_core::project::ProjectId,
    previous_state: ygg_core::project::ProjectState,
    new_state: ygg_core::project::ProjectState,
    #[schemars(schema_with = "string_schema")]
    session_id: Option<String>,
}
#[derive(JsonSchema)]
struct ProjectStatusResult {
    project_id: ygg_core::project::ProjectId,
    state: ygg_core::project::ProjectState,
    sessions_count: usize,
    secrets_count: usize,
    #[schemars(schema_with = "string_schema")]
    running_session_id: Option<String>,
    storage_summary: Option<ProjectStorageSummarySchema>,
}
#[derive(JsonSchema)]
struct ProjectLifecyclePayloadSchema {
    project_id: ygg_core::project::ProjectId,
    title: String,
    #[serde(rename = "type")]
    project_type: ygg_core::project::ProjectType,
    previous_state: Option<ygg_core::project::ProjectState>,
    new_state: ygg_core::project::ProjectState,
}
#[derive(JsonSchema)]
struct AssetGetParams {
    asset_id: String,
}
#[derive(JsonSchema)]
struct ProjectionIdParams {
    projection_id: String,
}
#[derive(JsonSchema)]
struct ProposalIdParams {
    proposal_id: String,
}
#[derive(JsonSchema)]
struct ProposalDecisionParams {
    proposal_id: String,
    reason: Option<String>,
}
#[derive(JsonSchema)]
struct SurfaceListParams {
    slot: Option<String>,
}
#[derive(JsonSchema)]
struct SurfaceDescribeParams {
    surface_id: String,
}
#[derive(JsonSchema)]
struct SurfaceResolveBundleParams {
    surface_id: String,
}
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum SurfaceBundleSourceSchema {
    InstalledProject,
    DevPath,
}
#[derive(JsonSchema)]
struct SurfaceResolveBundleResult {
    surface_id: String,
    bundle_url: String,
    export_name: String,
    stylesheets: Vec<String>,
    wrapper_class: Option<String>,
    project_id: Option<String>,
    source: SurfaceBundleSourceSchema,
}
#[derive(JsonSchema)]
struct PermissionGrantParams {
    principal: ProtocolPrincipal,
    permission: String,
    scope: Option<String>,
    reason: Option<String>,
}
#[derive(JsonSchema)]
struct PermissionRevokeParams {
    grant_id: String,
}
#[derive(JsonSchema)]
struct PermissionListParams {
    principal: Option<ProtocolPrincipal>,
}
#[derive(JsonSchema)]
struct OutboundAuditParams {
    package_id: String,
}
#[derive(JsonSchema)]
struct AuditPackageParams {
    package_id: PackageId,
    since: Option<chrono::DateTime<chrono::Utc>>,
    until: Option<chrono::DateTime<chrono::Utc>>,
}
#[derive(JsonSchema)]
struct CapAttenuateParams {
    parent_handle: CapHandleId,
    #[schemars(schema_with = "json_value_schema")]
    constraints: Value,
}
#[derive(JsonSchema)]
struct CapRevokeParams {
    handle: CapHandleId,
}
#[derive(JsonSchema)]
struct CapListForParams {
    package_id: PackageId,
}
#[derive(JsonSchema)]
struct CapHandleResult {
    handle: CapHandle,
}
#[derive(JsonSchema)]
struct CapHandlesResult {
    handles: Vec<CapHandle>,
}
#[derive(JsonSchema)]
struct CapabilityStreamParams {
    capability_id: String,
    provider_package_id: Option<String>,
    #[schemars(schema_with = "json_value_schema")]
    input: Value,
}
#[derive(JsonSchema)]
struct CapabilityCancelParams {
    stream_id: Option<String>,
    invocation_id: Option<String>,
    session_id: Option<String>,
}
#[derive(JsonSchema)]
struct OutboundExecuteParams {
    package_id: Option<String>,
    capability_id: String,
    destination_host: String,
    method: String,
    path: Option<String>,
    purpose: Option<String>,
    secret_refs: Vec<String>,
    timeout_ms: Option<u64>,
    #[schemars(schema_with = "json_value_schema")]
    metadata: Value,
    #[schemars(schema_with = "optional_json_value_schema")]
    body_shape: Option<Value>,
    secret_headers: Vec<OutboundSecretHeaderSpec>,
    static_headers: Vec<OutboundStaticHeader>,
}
#[derive(JsonSchema)]
struct OutboundStreamParams {
    package_id: Option<String>,
    capability_id: String,
    destination_host: String,
    method: String,
    path: Option<String>,
    purpose: Option<String>,
    secret_refs: Vec<String>,
    timeout_ms: Option<u64>,
    stream_format: Option<StreamFormat>,
    max_frame_bytes: Option<u64>,
    max_total_bytes: Option<u64>,
    max_duration_ms: Option<u64>,
    #[schemars(schema_with = "json_value_schema")]
    metadata: Value,
    #[schemars(schema_with = "optional_json_value_schema")]
    body_shape: Option<Value>,
    secret_headers: Vec<OutboundSecretHeaderSpec>,
    static_headers: Vec<OutboundStaticHeader>,
}
#[derive(JsonSchema)]
struct OutboundWebSocketSendParams {
    connection_id: String,
    kind: Option<FrameKind>,
    data: Option<String>,
    bytes: Option<Vec<u8>>,
    code: Option<u16>,
    reason: Option<String>,
}

fn json_value_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    schemars::schema::Schema::Bool(true)
}
fn optional_json_value_schema(
    _gen: &mut schemars::gen::SchemaGenerator,
) -> schemars::schema::Schema {
    schemars::schema::Schema::Bool(true)
}

fn string_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};

    let mut schema = SchemaObject::default();
    schema.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::String)));
    Schema::Object(schema)
}

fn schema_value<T: JsonSchema>() -> Value {
    let schema = schema_for!(T);
    let mut value = serde_json::to_value(schema).expect("schema serializes");
    normalize_schema(&mut value);
    value
}

fn normalize_schema(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.get("$schema").and_then(Value::as_str)
                == Some("http://json-schema.org/draft-07/schema#")
            {
                map.insert("$schema".to_string(), Value::String(SCHEMA.to_string()));
            }
            if let Some(defs) = map.remove("definitions") {
                map.insert("$defs".to_string(), defs);
            }
            for v in map.values_mut() {
                normalize_schema(v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                normalize_schema(v);
            }
        }
        _ => {}
    }
}

fn method_schema(method: KernelMethod, params: Value, result: Value) -> Value {
    json!({
        "$schema": SCHEMA,
        "$id": format!("{BASE}/methods/{}.schema.json", method.id()),
        "title": method.id(),
        "description": format!("Yggdrasil public kernel method {}.", method.id()),
        "type": "object",
        "additionalProperties": false,
        "required": ["method", "params"],
        "properties": {
            "method": { "const": method.id() },
            "params": { "$ref": "#/$defs/Params" },
            "result": { "$ref": "#/$defs/Result" },
            "errors": { "$ref": "#/$defs/Errors" }
        },
        "$defs": { "Params": params, "Result": result, "Errors": { "type": "array", "items": schema_value::<ErrorShape>() } }
    })
}

fn event_schema(kind: &str, payload: Value) -> Value {
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

fn filename(name: &str) -> String {
    name.replace('/', "__")
}

fn write_json(path: impl AsRef<Path>, value: &Value) -> anyhow::Result<()> {
    let mut value = value.clone();
    normalize_schema(&mut value);
    let bytes = serde_json::to_vec_pretty(&value)?;
    fs::write(path, [bytes, b"\n".to_vec()].concat())?;
    Ok(())
}

fn write_method(
    out: &Path,
    method: KernelMethod,
    params: Value,
    result: Value,
) -> anyhow::Result<()> {
    write_json(
        out.join("methods")
            .join(format!("{}.schema.json", method.id())),
        &method_schema(method, params, result),
    )
}

fn main() -> anyhow::Result<()> {
    let out = PathBuf::from("docs/spec/v1/schemas");
    fs::create_dir_all(out.join("methods"))?;
    fs::create_dir_all(out.join("events"))?;

    write_json(
        out.join("manifest.schema.json"),
        &schema_value::<PackageManifest>(),
    )?;
    write_json(
        out.join("capability-descriptor.schema.json"),
        &schema_value::<CapabilityDescriptor>(),
    )?;
    write_json(
        out.join("permission-set.schema.json"),
        &schema_value::<PermissionSet>(),
    )?;
    write_json(
        out.join("event-envelope.schema.json"),
        &schema_value::<EventEnvelope>(),
    )?;
    write_json(
        out.join("protocol-context.schema.json"),
        &schema_value::<ProtocolContext>(),
    )?;
    write_json(
        out.join("capability-invocation-request.schema.json"),
        &schema_value::<CapabilityInvocationRequest>(),
    )?;
    write_json(
        out.join("capability-invocation-result.schema.json"),
        &schema_value::<CapabilityInvocationResult>(),
    )?;

    for method in KernelMethod::all() {
        let (params, result) = match method {
            KernelMethod::SessionOpen => (
                schema_value::<OpenSessionRequest>(),
                schema_value::<KernelSession>(),
            ),
            KernelMethod::SessionClose => (
                schema_value::<SessionCloseParams>(),
                schema_value::<EventEnvelope>(),
            ),
            KernelMethod::SessionGet => (
                schema_value::<SessionGetParams>(),
                schema_value::<KernelSession>(),
            ),
            KernelMethod::SessionFork => (
                schema_value::<SessionForkParams>(),
                schema_value::<BranchRecord>(),
            ),
            KernelMethod::SessionBranchList => (
                schema_value::<SessionBranchListParams>(),
                json!({"type":"array","items":schema_value::<BranchRecord>()}),
            ),
            KernelMethod::SessionList
            | KernelMethod::EventSubscribe
            | KernelMethod::PackageDescribe
            | KernelMethod::CapabilityDescribe
            | KernelMethod::ExtensionPointDescribe
            | KernelMethod::HostPrincipal => {
                (schema_value::<EmptyParams>(), json!({"type":"null"}))
            }
            KernelMethod::EventAppend => (
                schema_value::<AppendEventRequest>(),
                schema_value::<EventEnvelope>(),
            ),
            KernelMethod::EventList => (
                schema_value::<EventListRequest>(),
                json!({"type":"array","items":schema_value::<EventEnvelope>()}),
            ),
            KernelMethod::PackageLoad => (
                schema_value::<PackageManifest>(),
                schema_value::<PackageRecord>(),
            ),
            KernelMethod::PackageUnload
            | KernelMethod::PackageRestart
            | KernelMethod::PackageLogs
            | KernelMethod::PackageStatus => {
                let result = if *method == KernelMethod::PackageLogs {
                    json!({"type":"array","items":schema_value::<SubprocessLogLine>()})
                } else {
                    schema_value::<PackageRecord>()
                };
                (schema_value::<PackageIdParams>(), result)
            }
            KernelMethod::PackageList => (
                schema_value::<EmptyParams>(),
                json!({"type":"array","items":schema_value::<PackageRecord>()}),
            ),
            KernelMethod::ProjectList => (
                schema_value::<ProjectListParams>(),
                schema_value::<ProjectListResultSchema>(),
            ),
            KernelMethod::ProjectGet => (
                schema_value::<ProjectIdParams>(),
                json!({"allOf":[schema_value::<ygg_core::project::ProjectDescriptor>()],"properties":{"state":schema_value::<ygg_core::project::ProjectState>(),"storage_summary":schema_value::<ProjectStorageSummarySchema>(),"running_session_id":{"type":"string","description":"Session id when project state is running; absent otherwise"}}}),
            ),
            KernelMethod::ProjectStart => (
                schema_value::<ProjectIdParams>(),
                schema_value::<ProjectStartResult>(),
            ),
            KernelMethod::ProjectStop => (
                schema_value::<ProjectIdParams>(),
                schema_value::<ProjectStopResult>(),
            ),
            KernelMethod::ProjectStatus => (
                schema_value::<ProjectIdParams>(),
                schema_value::<ProjectStatusResult>(),
            ),
            KernelMethod::CapabilityDiscover => (
                schema_value::<EmptyParams>(),
                json!({"type":"array","items":schema_value::<RegisteredCapability>()}),
            ),
            KernelMethod::CapabilityInvoke => (
                schema_value::<CapabilityInvocationRequest>(),
                schema_value::<CapabilityInvocationResult>(),
            ),
            KernelMethod::CapabilityHandleAttenuate => (
                schema_value::<CapAttenuateParams>(),
                schema_value::<CapHandleResult>(),
            ),
            KernelMethod::CapabilityHandleRevoke => {
                (schema_value::<CapRevokeParams>(), json!({"type":"object"}))
            }
            KernelMethod::CapabilityHandleListFor => (
                schema_value::<CapListForParams>(),
                schema_value::<CapHandlesResult>(),
            ),
            KernelMethod::CapabilityStream => (
                schema_value::<CapabilityStreamParams>(),
                json!({"type":"object"}),
            ),
            KernelMethod::CapabilityCancel => (
                schema_value::<CapabilityCancelParams>(),
                schema_value::<StreamFrameEnvelope>(),
            ),
            KernelMethod::ExtensionPointList => (
                schema_value::<EmptyParams>(),
                json!({"type":"array","items":{"type":"string"}}),
            ),
            KernelMethod::HookList => (
                schema_value::<EmptyParams>(),
                json!({"type":"array","items":schema_value::<RegisteredHook>()}),
            ),
            KernelMethod::AssetPut => (
                schema_value::<AssetPutRequest>(),
                schema_value::<AssetRecord>(),
            ),
            KernelMethod::AssetGet => (
                schema_value::<AssetGetParams>(),
                schema_value::<AssetGetResponse>(),
            ),
            KernelMethod::AssetList => (
                schema_value::<EmptyParams>(),
                json!({"type":"array","items":schema_value::<AssetRecord>()}),
            ),
            KernelMethod::ProjectionRegister => (
                schema_value::<ProjectionDefinition>(),
                schema_value::<ProjectionDefinition>(),
            ),
            KernelMethod::ProjectionRebuild | KernelMethod::ProjectionGet => (
                schema_value::<ProjectionIdParams>(),
                schema_value::<ProjectionDefinition>(),
            ),
            KernelMethod::ProjectionList => (
                schema_value::<EmptyParams>(),
                json!({"type":"array","items":schema_value::<ProjectionDefinition>()}),
            ),
            KernelMethod::HostInfo => (schema_value::<EmptyParams>(), schema_value::<HostInfo>()),
            KernelMethod::HostPing => (
                schema_value::<EmptyParams>(),
                json!({"type":"object","required":["ok"],"properties":{"ok":{"const":true}}}),
            ),
            KernelMethod::HostDiagnostics => {
                (schema_value::<EmptyParams>(), json!({"type":"object"}))
            }
            KernelMethod::PermissionGrant => (
                schema_value::<PermissionGrantParams>(),
                schema_value::<PermissionGrantRecord>(),
            ),
            KernelMethod::PermissionRevoke => (
                schema_value::<PermissionRevokeParams>(),
                schema_value::<PermissionGrantRecord>(),
            ),
            KernelMethod::PermissionList => (
                schema_value::<PermissionListParams>(),
                json!({"type":"array","items":schema_value::<PermissionGrantRecord>()}),
            ),
            KernelMethod::PermissionAudit => (
                schema_value::<EmptyParams>(),
                json!({"type":"array","items":schema_value::<EventEnvelope>()}),
            ),
            KernelMethod::AuditPackage => (
                schema_value::<AuditPackageParams>(),
                schema_value::<PackageAuditReport>(),
            ),
            KernelMethod::ProposalCreate => (
                schema_value::<ProposalRecord>(),
                schema_value::<ProposalRecord>(),
            ),
            KernelMethod::ProposalGet | KernelMethod::ProposalApply => (
                schema_value::<ProposalIdParams>(),
                schema_value::<ProposalRecord>(),
            ),
            KernelMethod::ProposalList => (
                schema_value::<EmptyParams>(),
                json!({"type":"array","items":schema_value::<ProposalRecord>()}),
            ),
            KernelMethod::ProposalApprove | KernelMethod::ProposalReject => (
                schema_value::<ProposalDecisionParams>(),
                schema_value::<ProposalRecord>(),
            ),
            KernelMethod::SurfaceContributionList => {
                (schema_value::<SurfaceListParams>(), json!({"type":"array"}))
            }
            KernelMethod::SurfaceResolveBundle => (
                schema_value::<SurfaceResolveBundleParams>(),
                schema_value::<SurfaceResolveBundleResult>(),
            ),
            KernelMethod::SurfaceContributionDescribe => (
                schema_value::<SurfaceDescribeParams>(),
                json!({"type":"object"}),
            ),
            KernelMethod::OutboundAudit => (
                schema_value::<OutboundAuditParams>(),
                json!({"type":"array","items":schema_value::<OutboundAuditRecord>()}),
            ),
            KernelMethod::OutboundExecute => (
                schema_value::<OutboundExecuteParams>(),
                schema_value::<OutboundExecutorResponse>(),
            ),
            KernelMethod::OutboundStream => (
                schema_value::<OutboundStreamParams>(),
                schema_value::<KernelOutboundStreamResponse>(),
            ),
            KernelMethod::OutboundWebSocketOpen => (
                schema_value::<OutboundWebSocketOpenRequest>(),
                json!({"type":"object"}),
            ),
            KernelMethod::OutboundWebSocketSend | KernelMethod::OutboundWebSocketClose => (
                schema_value::<OutboundWebSocketSendParams>(),
                json!({"type":"object"}),
            ),
        };
        write_method(&out, *method, params, result)?;
    }

    let events: Vec<(&str, Value)> = vec![
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
    ];
    for (kind, payload) in events {
        write_json(
            out.join("events")
                .join(format!("{}.schema.json", filename(kind))),
            &event_schema(kind, payload),
        )?;
    }
    Ok(())
}
