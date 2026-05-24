use schemars::{schema_for, JsonSchema};
use serde_json::Value;
use ygg_core::*;
use ygg_runtime::*;

use super::SCHEMA;

#[derive(JsonSchema)]
pub(crate) struct EmptyParams {}

#[derive(JsonSchema)]
pub(crate) struct ErrorShape {
    code: String,
    message: String,
    details: Value,
}

#[derive(JsonSchema)]
pub(crate) struct SessionCloseParams {
    session_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct SessionGetParams {
    session_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct SessionForkParams {
    parent_session_id: String,
    forked_from_sequence: u64,
    #[schemars(schema_with = "json_value_schema")]
    metadata: Value,
}
#[derive(JsonSchema)]
pub(crate) struct SessionBranchListParams {
    session_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct PackageIdParams {
    package_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectIdParams {
    project_id: ygg_core::project::ProjectId,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectListParams {
    filter_state: Option<ygg_core::project::ProjectState>,
}
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StorageMeasurementStateSchema {
    Measured,
    Unknown,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectStorageSummarySchema {
    data_bytes: Option<u64>,
    cache_bytes: Option<u64>,
    bundle_bytes: Option<u64>,
    log_bytes: Option<u64>,
    total_bytes: Option<u64>,
    measured_at: Option<chrono::DateTime<chrono::Utc>>,
    measurement_state: StorageMeasurementStateSchema,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectListItemSchema {
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
pub(crate) struct ProjectListResultSchema {
    projects: Vec<ProjectListItemSchema>,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectStartResult {
    project_id: ygg_core::project::ProjectId,
    previous_state: ygg_core::project::ProjectState,
    new_state: ygg_core::project::ProjectState,
    session_id: String,
    already_running: bool,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectStopResult {
    project_id: ygg_core::project::ProjectId,
    previous_state: ygg_core::project::ProjectState,
    new_state: ygg_core::project::ProjectState,
    #[schemars(schema_with = "string_schema")]
    session_id: Option<String>,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectStatusResult {
    project_id: ygg_core::project::ProjectId,
    state: ygg_core::project::ProjectState,
    sessions_count: usize,
    secrets_count: usize,
    #[schemars(schema_with = "string_schema")]
    running_session_id: Option<String>,
    storage_summary: Option<ProjectStorageSummarySchema>,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectLifecyclePayloadSchema {
    project_id: ygg_core::project::ProjectId,
    title: String,
    #[serde(rename = "type")]
    project_type: ygg_core::project::ProjectType,
    previous_state: Option<ygg_core::project::ProjectState>,
    new_state: ygg_core::project::ProjectState,
}
#[derive(JsonSchema)]
pub(crate) struct AssetGetParams {
    asset_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct ProjectionIdParams {
    projection_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct ProposalIdParams {
    proposal_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct ProposalDecisionParams {
    proposal_id: String,
    reason: Option<String>,
}
#[derive(JsonSchema)]
pub(crate) struct SurfaceListParams {
    slot: Option<String>,
}
#[derive(JsonSchema)]
pub(crate) struct SurfaceDescribeParams {
    surface_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct SurfaceResolveBundleParams {
    surface_id: String,
}
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SurfaceBundleSourceSchema {
    InstalledProject,
    DevPath,
}
#[derive(JsonSchema)]
pub(crate) struct SurfaceResolveBundleResult {
    surface_id: String,
    bundle_url: String,
    export_name: String,
    stylesheets: Vec<String>,
    wrapper_class: Option<String>,
    project_id: Option<String>,
    source: SurfaceBundleSourceSchema,
}
#[derive(JsonSchema)]
pub(crate) struct PermissionGrantParams {
    principal: ProtocolPrincipal,
    permission: String,
    scope: Option<String>,
    reason: Option<String>,
}
#[derive(JsonSchema)]
pub(crate) struct PermissionRevokeParams {
    grant_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct PermissionListParams {
    principal: Option<ProtocolPrincipal>,
}
#[derive(JsonSchema)]
pub(crate) struct OutboundAuditParams {
    package_id: String,
}
#[derive(JsonSchema)]
pub(crate) struct AuditPackageParams {
    package_id: PackageId,
    since: Option<chrono::DateTime<chrono::Utc>>,
    until: Option<chrono::DateTime<chrono::Utc>>,
}
#[derive(JsonSchema)]
pub(crate) struct CapAttenuateParams {
    parent_handle: CapHandleId,
    #[schemars(schema_with = "json_value_schema")]
    constraints: Value,
}
#[derive(JsonSchema)]
pub(crate) struct CapRevokeParams {
    handle: CapHandleId,
}
#[derive(JsonSchema)]
pub(crate) struct CapListForParams {
    package_id: PackageId,
}
#[derive(JsonSchema)]
pub(crate) struct CapHandleResult {
    handle: CapHandle,
}
#[derive(JsonSchema)]
pub(crate) struct CapHandlesResult {
    handles: Vec<CapHandle>,
}
#[derive(JsonSchema)]
pub(crate) struct CapabilityStreamParams {
    capability_id: String,
    provider_package_id: Option<String>,
    #[schemars(schema_with = "json_value_schema")]
    input: Value,
}
#[derive(JsonSchema)]
pub(crate) struct CapabilityCancelParams {
    stream_id: Option<String>,
    invocation_id: Option<String>,
    session_id: Option<String>,
}
#[derive(JsonSchema)]
pub(crate) struct OutboundExecuteParams {
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
pub(crate) struct OutboundStreamParams {
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
pub(crate) struct OutboundWebSocketSendParams {
    connection_id: String,
    kind: Option<FrameKind>,
    data: Option<String>,
    bytes: Option<Vec<u8>>,
    code: Option<u16>,
    reason: Option<String>,
}

pub(crate) fn json_value_schema(
    _gen: &mut schemars::r#gen::SchemaGenerator,
) -> schemars::schema::Schema {
    schemars::schema::Schema::Bool(true)
}
pub(crate) fn optional_json_value_schema(
    _gen: &mut schemars::r#gen::SchemaGenerator,
) -> schemars::schema::Schema {
    schemars::schema::Schema::Bool(true)
}

pub(crate) fn string_schema(
    _gen: &mut schemars::r#gen::SchemaGenerator,
) -> schemars::schema::Schema {
    use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};

    let mut schema = SchemaObject::default();
    schema.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::String)));
    Schema::Object(schema)
}

pub(crate) fn schema_value<T: JsonSchema>() -> Value {
    let schema = schema_for!(T);
    let mut value = serde_json::to_value(schema).expect("schema serializes");
    normalize_schema(&mut value);
    value
}

pub(crate) fn normalize_schema(value: &mut Value) {
    normalize_schema_with_key(None, value);
}

fn normalize_schema_with_key(parent_key: Option<&str>, value: &mut Value) {
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
            if matches!(parent_key, Some("created_at" | "timestamp"))
                && map.get("format").and_then(Value::as_str) == Some("date-time")
            {
                map.remove("default");
            }
            for (key, v) in map.iter_mut() {
                normalize_schema_with_key(Some(key.as_str()), v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                normalize_schema_with_key(parent_key, v);
            }
        }
        _ => {}
    }
}
