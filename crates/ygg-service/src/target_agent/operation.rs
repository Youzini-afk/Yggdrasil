use super::*;

use axum::body::Body;
use serde_json::Value;
use tokio::io::AsyncReadExt;
use ygg_core::ProjectId;
use ygg_runtime::scan_effect_value_for_raw_secrets;

use crate::require_identity_project;

const OPERATION_JOURNAL_SESSION: &str = "host_control_target_operations";
const OPERATION_JOURNAL_EVENT: &str = "host/control/v1/target_operation.snapshot";
const DEFAULT_AUTHORITY_TTL_SECS: u64 = 5 * 60;
const MAX_AUTHORITY_TTL_SECS: u64 = 15 * 60;
const MAX_RECEIPT_OUTPUT_BYTES: usize = 256 * 1024;
const MAX_RECEIPT_DIAGNOSTICS: usize = 32;
const MAX_RECEIPT_DIAGNOSTIC_BYTES: usize = 2 * 1024;
const OPERATION_STEP_ID: &str = "execute";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetOperationEffect {
    ArtifactMaterialize,
    ArtifactRelease,
    HealthProbe,
    VerifierRun,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeclarativeVerifierDescriptor {
    ArtifactIntegrity {
        digest: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_size_bytes: Option<u64>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TargetOperationSpec {
    ArtifactMaterialize {
        digest: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_size_bytes: Option<u64>,
    },
    ArtifactRelease {
        digest: String,
    },
    HealthProbe,
    VerifierRun {
        verifier: DeclarativeVerifierDescriptor,
    },
}

impl TargetOperationSpec {
    pub fn effect(&self) -> TargetOperationEffect {
        match self {
            Self::ArtifactMaterialize { .. } => TargetOperationEffect::ArtifactMaterialize,
            Self::ArtifactRelease { .. } => TargetOperationEffect::ArtifactRelease,
            Self::HealthProbe => TargetOperationEffect::HealthProbe,
            Self::VerifierRun { .. } => TargetOperationEffect::VerifierRun,
        }
    }

    pub fn artifact_digests(&self) -> Vec<String> {
        match self {
            Self::ArtifactMaterialize { digest, .. } | Self::ArtifactRelease { digest } => {
                vec![digest.clone()]
            }
            Self::HealthProbe => Vec::new(),
            Self::VerifierRun {
                verifier: DeclarativeVerifierDescriptor::ArtifactIntegrity { digest, .. },
            } => vec![digest.clone()],
        }
    }

    fn expected_size(&self, candidate_digest: &str) -> Option<u64> {
        match self {
            Self::ArtifactMaterialize {
                digest,
                expected_size_bytes,
            }
            | Self::VerifierRun {
                verifier:
                    DeclarativeVerifierDescriptor::ArtifactIntegrity {
                        digest,
                        expected_size_bytes,
                    },
            } if digest == candidate_digest => *expected_size_bytes,
            _ => None,
        }
    }

    fn validate(&self) -> Result<(), ServiceError> {
        for digest in self.artifact_digests() {
            validate_sha256_digest(&digest)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TargetOperationAuthority {
    pub target_id: String,
    pub operation_id: String,
    pub step_id: String,
    pub project_id: ProjectId,
    pub effect: TargetOperationEffect,
    pub artifact_digests: Vec<String>,
    pub lease_epoch: u64,
    pub policy_epoch: u64,
    pub issued_at_ms: i64,
    pub expires_at_ms: i64,
    pub nonce: String,
    pub request_digest: String,
    pub authority_digest: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetOperationStatusKind {
    Requested,
    Accepted,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    OutcomeUnknown,
    Expired,
}

impl TargetOperationStatusKind {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::OutcomeUnknown | Self::Expired
        )
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetOperationReceiptStatus {
    Succeeded,
    Failed,
    Cancelled,
    OutcomeUnknown,
}

impl From<TargetOperationReceiptStatus> for TargetOperationStatusKind {
    fn from(value: TargetOperationReceiptStatus) -> Self {
        match value {
            TargetOperationReceiptStatus::Succeeded => Self::Succeeded,
            TargetOperationReceiptStatus::Failed => Self::Failed,
            TargetOperationReceiptStatus::Cancelled => Self::Cancelled,
            TargetOperationReceiptStatus::OutcomeUnknown => Self::OutcomeUnknown,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TargetOperationReceipt {
    pub operation_id: String,
    pub target_id: String,
    pub execution_id: String,
    pub step_id: String,
    pub request_digest: String,
    pub authority_digest: String,
    pub status: TargetOperationReceiptStatus,
    pub completed_at_ms: i64,
    #[serde(default)]
    pub output: Value,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TargetOperationRecord {
    pub operation_id: String,
    pub target_id: String,
    pub project_id: ProjectId,
    pub revision: u64,
    pub status: TargetOperationStatusKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    pub spec: TargetOperationSpec,
    pub authority: TargetOperationAuthority,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<TargetOperationReceipt>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTargetOperationRequest {
    pub project_id: ProjectId,
    pub spec: TargetOperationSpec,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub expires_in_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateTargetOperationResponse {
    pub operation: TargetOperationRecord,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NextTargetOperationResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<TargetOperationRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TargetOperationProgressRequest {
    pub request_digest: String,
    pub authority_digest: String,
    pub execution_id: String,
    pub status: TargetOperationStatusKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TargetOperationSnapshot {
    record: TargetOperationRecord,
}

#[derive(Debug, Default)]
pub(super) struct TargetOperationState {
    next_sequence: EventSequence,
    operations: HashMap<String, TargetOperationRecord>,
    idempotency: HashMap<(String, ProjectId, String), (String, String)>,
}

impl TargetAgentRegistry {
    fn operation_next_sequence(&self) -> EventSequence {
        self.operations
            .lock()
            .expect("target operation state lock poisoned")
            .next_sequence
    }

    fn operation(&self, operation_id: &str) -> Option<TargetOperationRecord> {
        self.operations
            .lock()
            .expect("target operation state lock poisoned")
            .operations
            .get(operation_id)
            .cloned()
    }

    fn operations_for_target(&self, target_id: &str) -> Vec<TargetOperationRecord> {
        let mut operations = self
            .operations
            .lock()
            .expect("target operation state lock poisoned")
            .operations
            .values()
            .filter(|record| record.target_id == target_id)
            .cloned()
            .collect::<Vec<_>>();
        operations.sort_by(|left, right| {
            left.created_at_ms
                .cmp(&right.created_at_ms)
                .then_with(|| left.operation_id.cmp(&right.operation_id))
        });
        operations
    }

    fn idempotent_operation(
        &self,
        target_id: &str,
        project_id: &ProjectId,
        key: &str,
    ) -> Option<(String, TargetOperationRecord)> {
        let state = self
            .operations
            .lock()
            .expect("target operation state lock poisoned");
        let (request_digest, operation_id) =
            state
                .idempotency
                .get(&(target_id.to_string(), project_id.clone(), key.to_string()))?;
        state
            .operations
            .get(operation_id)
            .cloned()
            .map(|record| (request_digest.clone(), record))
    }

    fn apply_operation_event(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
        anyhow::ensure!(
            envelope.session_id == OPERATION_JOURNAL_SESSION
                && envelope.kind == OPERATION_JOURNAL_EVENT,
            "invalid target operation journal envelope"
        );
        let snapshot: TargetOperationSnapshot = serde_json::from_value(envelope.payload.clone())?;
        let record = snapshot.record;
        let authority_key = self
            .operation_authority_key(&record.target_id, record.authority.lease_epoch)
            .ok_or_else(|| anyhow::anyhow!("target operation authority key is unknown"))?;
        validate_record_integrity(&record, &authority_key)?;

        let mut state = self
            .operations
            .lock()
            .expect("target operation state lock poisoned");
        if envelope.sequence < state.next_sequence {
            return Ok(());
        }
        anyhow::ensure!(
            envelope.sequence == state.next_sequence,
            "target operation journal sequence is not contiguous"
        );

        if let Some(previous) = state.operations.get(&record.operation_id) {
            anyhow::ensure!(
                record.revision == previous.revision.saturating_add(1)
                    && immutable_operation_fields_match(previous, &record)
                    && valid_execution_owner_transition(previous, &record)
                    && record.updated_at_ms >= previous.updated_at_ms
                    && valid_status_transition(previous.status, record.status),
                "target operation snapshot transition is invalid"
            );
        } else {
            anyhow::ensure!(
                record.revision == 1
                    && record.status == TargetOperationStatusKind::Requested
                    && record.execution_id.is_none()
                    && record.receipt.is_none(),
                "new target operation snapshot is invalid"
            );
        }

        if let Some(key) = record.idempotency_key.as_deref() {
            let index_key = (
                record.target_id.clone(),
                record.project_id.clone(),
                key.to_string(),
            );
            if let Some((request_digest, operation_id)) = state.idempotency.get(&index_key) {
                anyhow::ensure!(
                    request_digest == &record.authority.request_digest
                        && operation_id == &record.operation_id,
                    "target operation idempotency key was reused"
                );
            } else {
                state.idempotency.insert(
                    index_key,
                    (
                        record.authority.request_digest.clone(),
                        record.operation_id.clone(),
                    ),
                );
            }
        }
        state.operations.insert(record.operation_id.clone(), record);
        state.next_sequence = envelope.sequence.saturating_add(1);
        Ok(())
    }
}

fn immutable_operation_fields_match(
    previous: &TargetOperationRecord,
    next: &TargetOperationRecord,
) -> bool {
    previous.operation_id == next.operation_id
        && previous.target_id == next.target_id
        && previous.project_id == next.project_id
        && previous.spec == next.spec
        && previous.authority == next.authority
        && previous.idempotency_key == next.idempotency_key
        && previous.created_at_ms == next.created_at_ms
}

fn valid_execution_owner_transition(
    previous: &TargetOperationRecord,
    next: &TargetOperationRecord,
) -> bool {
    if previous.status == TargetOperationStatusKind::Requested {
        previous.execution_id.is_none()
            && match next.status {
                TargetOperationStatusKind::Accepted => next.execution_id.is_some(),
                TargetOperationStatusKind::Expired => next.execution_id.is_none(),
                _ => false,
            }
    } else {
        previous.execution_id.is_some() && previous.execution_id == next.execution_id
    }
}

fn valid_status_transition(
    previous: TargetOperationStatusKind,
    next: TargetOperationStatusKind,
) -> bool {
    matches!(
        (previous, next),
        (
            TargetOperationStatusKind::Requested,
            TargetOperationStatusKind::Accepted | TargetOperationStatusKind::Expired
        ) | (
            TargetOperationStatusKind::Accepted,
            TargetOperationStatusKind::Running
                | TargetOperationStatusKind::Succeeded
                | TargetOperationStatusKind::Failed
                | TargetOperationStatusKind::Cancelled
                | TargetOperationStatusKind::OutcomeUnknown
        ) | (
            TargetOperationStatusKind::Running,
            TargetOperationStatusKind::Succeeded
                | TargetOperationStatusKind::Failed
                | TargetOperationStatusKind::Cancelled
                | TargetOperationStatusKind::OutcomeUnknown
        )
    )
}

#[derive(Serialize)]
struct OperationRequestDigestInput<'a> {
    target_id: &'a str,
    project_id: &'a ProjectId,
    step_id: &'a str,
    spec: &'a TargetOperationSpec,
}

#[derive(Serialize)]
struct UnsignedOperationAuthority<'a> {
    target_id: &'a str,
    operation_id: &'a str,
    step_id: &'a str,
    project_id: &'a ProjectId,
    effect: TargetOperationEffect,
    artifact_digests: &'a [String],
    lease_epoch: u64,
    policy_epoch: u64,
    issued_at_ms: i64,
    expires_at_ms: i64,
    nonce: &'a str,
    request_digest: &'a str,
}

fn digest_serializable<T>(domain: &str, value: &T) -> anyhow::Result<String>
where
    T: Serialize + ?Sized,
{
    let mut hasher = Sha256::new();
    hasher.update(b"yggdrasil-target-operation-v1\0");
    hasher.update(domain.as_bytes());
    hasher.update(b"\0");
    hasher.update(serde_json::to_vec(value)?);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn operation_request_digest(
    target_id: &str,
    project_id: &ProjectId,
    spec: &TargetOperationSpec,
) -> anyhow::Result<String> {
    digest_serializable(
        "request",
        &OperationRequestDigestInput {
            target_id,
            project_id,
            step_id: OPERATION_STEP_ID,
            spec,
        },
    )
}

fn operation_authority_digest(
    authority: &TargetOperationAuthority,
    authority_key: &str,
) -> anyhow::Result<String> {
    let mut message = b"yggdrasil-target-operation-authority-v1\0".to_vec();
    message.extend(serde_json::to_vec(&UnsignedOperationAuthority {
        target_id: &authority.target_id,
        operation_id: &authority.operation_id,
        step_id: &authority.step_id,
        project_id: &authority.project_id,
        effect: authority.effect,
        artifact_digests: &authority.artifact_digests,
        lease_epoch: authority.lease_epoch,
        policy_epoch: authority.policy_epoch,
        issued_at_ms: authority.issued_at_ms,
        expires_at_ms: authority.expires_at_ms,
        nonce: &authority.nonce,
        request_digest: &authority.request_digest,
    })?);
    Ok(format!(
        "sha256:{:x}",
        hmac_sha256(authority_key.as_bytes(), &message)
    ))
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> sha2::digest::Output<Sha256> {
    const BLOCK_BYTES: usize = 64;
    let mut key_block = [0u8; BLOCK_BYTES];
    if key.len() > BLOCK_BYTES {
        key_block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut inner_pad = [0x36u8; BLOCK_BYTES];
    let mut outer_pad = [0x5cu8; BLOCK_BYTES];
    for index in 0..BLOCK_BYTES {
        inner_pad[index] ^= key_block[index];
        outer_pad[index] ^= key_block[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner.finalize());
    outer.finalize()
}

fn validate_record_integrity(
    record: &TargetOperationRecord,
    authority_key: &str,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !record.operation_id.is_empty()
            && record.target_id == record.authority.target_id
            && record.operation_id == record.authority.operation_id
            && record.project_id == record.authority.project_id
            && record.authority.step_id == OPERATION_STEP_ID
            && record.spec.effect() == record.authority.effect
            && record.spec.artifact_digests() == record.authority.artifact_digests
            && record.authority.lease_epoch > 0
            && record.authority.policy_epoch > 0
            && record.authority.issued_at_ms < record.authority.expires_at_ms
            && record.created_at_ms == record.authority.issued_at_ms
            && record.updated_at_ms >= record.created_at_ms,
        "target operation authority fields are inconsistent"
    );
    anyhow::ensure!(
        match record.status {
            TargetOperationStatusKind::Requested | TargetOperationStatusKind::Expired => {
                record.execution_id.is_none()
            }
            _ => record.execution_id.as_deref().is_some_and(is_execution_id),
        },
        "target operation execution owner is invalid"
    );
    for digest in &record.authority.artifact_digests {
        anyhow::ensure!(is_sha256_digest(digest), "invalid artifact digest");
    }
    anyhow::ensure!(
        operation_request_digest(&record.target_id, &record.project_id, &record.spec)?
            == record.authority.request_digest,
        "target operation request digest did not match"
    );
    anyhow::ensure!(
        operation_authority_digest(&record.authority, authority_key)?
            == record.authority.authority_digest,
        "target operation authority MAC did not match"
    );

    match (record.status, record.receipt.as_ref()) {
        (TargetOperationStatusKind::Expired, None) => {}
        (status, Some(receipt)) if status.is_terminal() => {
            validate_receipt_for_record(record, receipt)?;
        }
        (status, None) if !status.is_terminal() => {}
        _ => anyhow::bail!("target operation receipt does not match terminal state"),
    }
    Ok(())
}

fn validate_receipt_for_record(
    record: &TargetOperationRecord,
    receipt: &TargetOperationReceipt,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        receipt.operation_id == record.operation_id
            && receipt.target_id == record.target_id
            && Some(receipt.execution_id.as_str()) == record.execution_id.as_deref()
            && receipt.step_id == OPERATION_STEP_ID
            && receipt.request_digest == record.authority.request_digest
            && receipt.authority_digest == record.authority.authority_digest
            && TargetOperationStatusKind::from(receipt.status) == record.status
            && receipt.completed_at_ms >= record.authority.issued_at_ms,
        "target operation receipt binding is invalid"
    );
    anyhow::ensure!(
        serde_json::to_vec(&receipt.output)?.len() <= MAX_RECEIPT_OUTPUT_BYTES,
        "target operation receipt output is too large"
    );
    anyhow::ensure!(
        !scan_effect_value_for_raw_secrets(&receipt.output, "receipt.output").has_findings(),
        "target operation receipt contains raw secret material"
    );
    anyhow::ensure!(
        receipt.diagnostics.len() <= MAX_RECEIPT_DIAGNOSTICS
            && receipt.diagnostics.iter().all(|diagnostic| {
                diagnostic.len() <= MAX_RECEIPT_DIAGNOSTIC_BYTES
                    && !scan_effect_value_for_raw_secrets(
                        &Value::String(diagnostic.clone()),
                        "receipt.diagnostics",
                    )
                    .has_findings()
                    && !diagnostic.chars().any(|character| {
                        character.is_control()
                            && character != '\n'
                            && character != '\r'
                            && character != '\t'
                    })
            }),
        "target operation receipt diagnostics exceed their limits"
    );
    Ok(())
}

fn is_execution_id(execution_id: &str) -> bool {
    execution_id.len() == 32
        && execution_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Validates the complete Host authority before a native agent accepts work.
/// `require_unexpired` must be true before first acceptance and may be false
/// only when a durable local ledger proves the same step was already accepted.
pub fn verify_target_operation_authority(
    record: &TargetOperationRecord,
    credential: &str,
    expected_target_id: &str,
    expected_lease_epoch: u64,
    expected_policy_epoch: u64,
    now_ms: i64,
    require_unexpired: bool,
) -> Result<(), String> {
    let authority_key = credential_digest("agent", credential);
    validate_record_integrity(record, &authority_key)
        .map_err(|_| "operation authority is malformed".to_string())?;
    if record.target_id != expected_target_id
        || record.authority.lease_epoch != expected_lease_epoch
        || record.authority.policy_epoch != expected_policy_epoch
        || now_ms < record.authority.issued_at_ms
        || (require_unexpired && now_ms >= record.authority.expires_at_ms)
    {
        return Err("operation authority audience, epoch, or expiry did not match".to_string());
    }
    Ok(())
}

fn is_sha256_digest(digest: &str) -> bool {
    digest.len() == 71
        && digest.starts_with("sha256:")
        && digest[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_sha256_digest(digest: &str) -> Result<(), ServiceError> {
    if is_sha256_digest(digest) {
        Ok(())
    } else {
        Err(ServiceError::with_status(
            StatusCode::BAD_REQUEST,
            "artifact digest must be sha256:<64 lowercase hexadecimal characters>",
        ))
    }
}

fn validate_create_request(
    target_id: &str,
    request: &CreateTargetOperationRequest,
) -> Result<(), ServiceError> {
    if !valid_target_id(target_id) || target_id == "local" {
        return Err(ServiceError::with_status(
            StatusCode::BAD_REQUEST,
            "target operation requires a valid enrolled remote target_id",
        ));
    }
    request.spec.validate()?;
    if let Some(key) = request.idempotency_key.as_deref() {
        if key.is_empty()
            || key.len() > 128
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._:".contains(&byte))
        {
            return Err(ServiceError::with_status(
                StatusCode::BAD_REQUEST,
                "idempotency_key must use 1..=128 label-safe ASCII characters",
            ));
        }
    }
    let ttl = request
        .expires_in_seconds
        .unwrap_or(DEFAULT_AUTHORITY_TTL_SECS);
    if ttl == 0 || ttl > MAX_AUTHORITY_TTL_SECS {
        return Err(ServiceError::with_status(
            StatusCode::BAD_REQUEST,
            "operation authority expiry is outside the supported range",
        ));
    }
    Ok(())
}

fn required_capabilities(spec: &TargetOperationSpec) -> &'static [ExecutionTargetCapability] {
    match spec {
        TargetOperationSpec::ArtifactMaterialize { .. }
        | TargetOperationSpec::ArtifactRelease { .. } => {
            &[ExecutionTargetCapability::ArtifactTransfer]
        }
        TargetOperationSpec::HealthProbe => &[ExecutionTargetCapability::HealthProbe],
        TargetOperationSpec::VerifierRun { .. } => &[
            ExecutionTargetCapability::ArtifactTransfer,
            ExecutionTargetCapability::DeclarativeVerifier,
        ],
    }
}

pub(super) fn protected_routes<S>() -> Router<AppState<S>>
where
    S: EventStore,
{
    Router::new()
        .route(
            "/host/v1/targets/:target_id/operations",
            post(create_operation::<S>).get(list_operations::<S>),
        )
        .route(
            "/host/v1/targets/:target_id/operations/:operation_id",
            get(get_operation::<S>),
        )
}

pub(super) fn agent_routes<S>() -> Router<AppState<S>>
where
    S: EventStore,
{
    Router::new()
        .route("/target-agent/v1/operations/next", get(next_operation::<S>))
        .route(
            "/target-agent/v1/operations/:operation_id/progress",
            post(progress_operation::<S>),
        )
        .route(
            "/target-agent/v1/operations/:operation_id/receipt",
            post(complete_operation::<S>),
        )
        .route(
            "/target-agent/v1/operations/:operation_id/artifacts/:digest",
            get(stream_operation_artifact::<S>),
        )
}

async fn create_operation<S>(
    State(state): State<AppState<S>>,
    Extension(identity): Extension<HostAccessIdentity>,
    Path(target_id): Path<String>,
    Json(request): Json<CreateTargetOperationRequest>,
) -> Result<(StatusCode, Json<CreateTargetOperationResponse>), ServiceError>
where
    S: EventStore,
{
    require_identity_target(&identity, &target_id)?;
    require_identity_project(&identity, request.project_id.as_str())?;
    validate_create_request(&target_id, &request)?;
    if state
        .runtime
        .config()
        .project_registry
        .get(&request.project_id)
        .is_none()
    {
        return Err(ServiceError::with_status(
            StatusCode::NOT_FOUND,
            "project is not registered",
        ));
    }
    sync_target_agent_journal(
        state.runtime.store().as_ref(),
        state.target_agents.as_ref(),
        state.runtime.config().target_registry.as_ref(),
    )
    .await
    .map_err(target_internal_error)?;
    let target = state
        .runtime
        .config()
        .target_registry
        .status(&target_id)
        .await
        .ok_or_else(|| ServiceError::with_status(StatusCode::NOT_FOUND, "target not found"))?;
    let authority_key = state
        .target_agents
        .operation_authority_key(&target.id, target.lease_epoch)
        .ok_or_else(|| {
            ServiceError::with_status(
                StatusCode::CONFLICT,
                "target identity has no operation authority key",
            )
        })?;
    if target.status != ExecutionTargetStatusKind::Available
        || required_capabilities(&request.spec)
            .iter()
            .any(|required| !target.capabilities.contains(required))
    {
        return Err(ServiceError::with_status(
            StatusCode::CONFLICT,
            "target is offline or lacks a required effective capability",
        ));
    }

    let operation = create_operation_record(
        state.runtime.store().as_ref(),
        state.target_agents.as_ref(),
        &authority_key,
        target,
        request,
    )
    .await
    .map_err(target_conflict_error)?;
    Ok((
        StatusCode::CREATED,
        Json(CreateTargetOperationResponse { operation }),
    ))
}

async fn list_operations<S>(
    State(state): State<AppState<S>>,
    Extension(identity): Extension<HostAccessIdentity>,
    Path(target_id): Path<String>,
) -> Result<Json<Vec<TargetOperationRecord>>, ServiceError>
where
    S: EventStore,
{
    require_identity_target(&identity, &target_id)?;
    sync_target_operation_journal(state.runtime.store().as_ref(), state.target_agents.as_ref())
        .await
        .map_err(target_internal_error)?;
    let operations = state
        .target_agents
        .operations_for_target(&target_id)
        .into_iter()
        .filter(|operation| identity.allows_project(operation.project_id.as_str()))
        .collect();
    Ok(Json(operations))
}

async fn get_operation<S>(
    State(state): State<AppState<S>>,
    Extension(identity): Extension<HostAccessIdentity>,
    Path((target_id, operation_id)): Path<(String, String)>,
) -> Result<Json<TargetOperationRecord>, ServiceError>
where
    S: EventStore,
{
    require_identity_target(&identity, &target_id)?;
    sync_target_operation_journal(state.runtime.store().as_ref(), state.target_agents.as_ref())
        .await
        .map_err(target_internal_error)?;
    let operation = state
        .target_agents
        .operation(&operation_id)
        .filter(|operation| operation.target_id == target_id)
        .ok_or_else(|| ServiceError::with_status(StatusCode::NOT_FOUND, "operation not found"))?;
    require_identity_project(&identity, operation.project_id.as_str())?;
    Ok(Json(operation))
}

async fn authenticated_agent<S>(
    state: &AppState<S>,
    headers: &HeaderMap,
) -> Result<StoredAgent, ServiceError>
where
    S: EventStore,
{
    let credential = target_credential(headers).ok_or_else(target_unauthorized)?;
    sync_target_agent_journal(
        state.runtime.store().as_ref(),
        state.target_agents.as_ref(),
        state.runtime.config().target_registry.as_ref(),
    )
    .await
    .map_err(target_internal_error)?;
    let agent = state
        .target_agents
        .authenticate_agent(credential)
        .ok_or_else(target_unauthorized)?;
    let live_target = state
        .runtime
        .config()
        .target_registry
        .status(&agent.target.id)
        .await
        .ok_or_else(target_unauthorized)?;
    if live_target.status != ExecutionTargetStatusKind::Available
        || live_target.identity_ref != agent.target.identity_ref
        || live_target.lease_epoch != agent.target.lease_epoch
        || live_target.policy_epoch != agent.target.policy_epoch
    {
        return Err(ServiceError::with_status(
            StatusCode::CONFLICT,
            "target must heartbeat before requesting operation work",
        ));
    }
    Ok(agent)
}

async fn next_operation<S>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
) -> Result<Json<NextTargetOperationResponse>, ServiceError>
where
    S: EventStore,
{
    let agent = authenticated_agent(&state, &headers).await?;
    'refresh: for _ in 0..8 {
        sync_target_operation_journal(state.runtime.store().as_ref(), state.target_agents.as_ref())
            .await
            .map_err(target_internal_error)?;
        let now_ms = Utc::now().timestamp_millis();
        for operation in state.target_agents.operations_for_target(&agent.target.id) {
            if operation.status.is_terminal() {
                continue;
            }
            let epoch_matches = operation.authority.lease_epoch == agent.target.lease_epoch
                && operation.authority.policy_epoch == agent.target.policy_epoch;
            if operation.status == TargetOperationStatusKind::Requested
                && (!epoch_matches || now_ms >= operation.authority.expires_at_ms)
            {
                let mut expired = operation;
                expired.revision = expired.revision.saturating_add(1);
                expired.status = TargetOperationStatusKind::Expired;
                expired.updated_at_ms = now_ms;
                if append_target_operation_snapshot(
                    state.runtime.store().as_ref(),
                    state.target_agents.as_ref(),
                    state.target_agents.operation_next_sequence(),
                    &expired,
                )
                .await
                .map_err(target_internal_error)?
                .is_some()
                {
                    continue 'refresh;
                }
                continue 'refresh;
            }
            if epoch_matches {
                return Ok(Json(NextTargetOperationResponse {
                    operation: Some(operation),
                }));
            }
        }
        return Ok(Json(NextTargetOperationResponse { operation: None }));
    }
    Err(ServiceError::with_status(
        StatusCode::CONFLICT,
        "target operation queue changed too frequently",
    ))
}

async fn progress_operation<S>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    Path(operation_id): Path<String>,
    Json(request): Json<TargetOperationProgressRequest>,
) -> Result<Json<TargetOperationRecord>, ServiceError>
where
    S: EventStore,
{
    if !matches!(
        request.status,
        TargetOperationStatusKind::Accepted | TargetOperationStatusKind::Running
    ) {
        return Err(ServiceError::with_status(
            StatusCode::BAD_REQUEST,
            "agent progress may only report accepted or running",
        ));
    }
    let agent = authenticated_agent(&state, &headers).await?;
    for _ in 0..8 {
        sync_target_operation_journal(state.runtime.store().as_ref(), state.target_agents.as_ref())
            .await
            .map_err(target_internal_error)?;
        let operation = state
            .target_agents
            .operation(&operation_id)
            .filter(|operation| operation.target_id == agent.target.id)
            .ok_or_else(|| {
                ServiceError::with_status(StatusCode::NOT_FOUND, "operation not found")
            })?;
        validate_agent_operation_binding(
            &operation,
            &agent,
            &request.request_digest,
            &request.authority_digest,
            Some(&request.execution_id),
        )?;
        if operation.status == request.status {
            return Ok(Json(operation));
        }
        let now_ms = Utc::now().timestamp_millis();
        if operation.status == TargetOperationStatusKind::Requested
            && (request.status != TargetOperationStatusKind::Accepted
                || now_ms >= operation.authority.expires_at_ms)
        {
            return Err(ServiceError::with_status(
                StatusCode::CONFLICT,
                "operation must be accepted before expiry and before it can run",
            ));
        }
        if operation.status == TargetOperationStatusKind::Accepted
            && request.status != TargetOperationStatusKind::Running
        {
            return Err(ServiceError::with_status(
                StatusCode::CONFLICT,
                "operation progress transition is invalid",
            ));
        }
        if operation.status.is_terminal()
            || !matches!(
                operation.status,
                TargetOperationStatusKind::Requested | TargetOperationStatusKind::Accepted
            )
        {
            return Err(ServiceError::with_status(
                StatusCode::CONFLICT,
                "operation no longer accepts progress",
            ));
        }
        let mut next = operation;
        next.revision = next.revision.saturating_add(1);
        next.status = request.status;
        if next.execution_id.is_none() {
            next.execution_id = Some(request.execution_id.clone());
        }
        next.updated_at_ms = now_ms;
        if append_target_operation_snapshot(
            state.runtime.store().as_ref(),
            state.target_agents.as_ref(),
            state.target_agents.operation_next_sequence(),
            &next,
        )
        .await
        .map_err(target_internal_error)?
        .is_some()
        {
            return Ok(Json(next));
        }
    }
    Err(ServiceError::with_status(
        StatusCode::CONFLICT,
        "target operation journal changed too frequently",
    ))
}

async fn complete_operation<S>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    Path(operation_id): Path<String>,
    Json(receipt): Json<TargetOperationReceipt>,
) -> Result<Json<TargetOperationRecord>, ServiceError>
where
    S: EventStore,
{
    if receipt.operation_id != operation_id {
        return Err(ServiceError::with_status(
            StatusCode::BAD_REQUEST,
            "receipt operation_id does not match the request path",
        ));
    }
    let now_ms = Utc::now().timestamp_millis();
    if receipt.completed_at_ms > now_ms.saturating_add(60_000) {
        return Err(ServiceError::with_status(
            StatusCode::BAD_REQUEST,
            "receipt completion time is too far in the future",
        ));
    }
    let agent = authenticated_agent(&state, &headers).await?;
    for _ in 0..8 {
        sync_target_operation_journal(state.runtime.store().as_ref(), state.target_agents.as_ref())
            .await
            .map_err(target_internal_error)?;
        let operation = state
            .target_agents
            .operation(&operation_id)
            .filter(|operation| operation.target_id == agent.target.id)
            .ok_or_else(|| {
                ServiceError::with_status(StatusCode::NOT_FOUND, "operation not found")
            })?;
        if operation.status.is_terminal() {
            if operation.receipt.as_ref() == Some(&receipt) {
                return Ok(Json(operation));
            }
            return Err(ServiceError::with_status(
                StatusCode::CONFLICT,
                "operation already has a different terminal receipt",
            ));
        }
        validate_agent_operation_binding(
            &operation,
            &agent,
            &receipt.request_digest,
            &receipt.authority_digest,
            Some(&receipt.execution_id),
        )?;
        if !matches!(
            operation.status,
            TargetOperationStatusKind::Accepted | TargetOperationStatusKind::Running
        ) {
            return Err(ServiceError::with_status(
                StatusCode::CONFLICT,
                "operation must be durably accepted before terminal receipt",
            ));
        }
        let mut next = operation;
        next.revision = next.revision.saturating_add(1);
        next.status = receipt.status.into();
        next.updated_at_ms = now_ms.max(receipt.completed_at_ms);
        next.receipt = Some(receipt.clone());
        let authority_key = state
            .target_agents
            .operation_authority_key(&next.target_id, next.authority.lease_epoch)
            .ok_or_else(|| {
                ServiceError::with_status(
                    StatusCode::CONFLICT,
                    "target operation authority key is no longer available",
                )
            })?;
        validate_record_integrity(&next, &authority_key).map_err(|_| {
            ServiceError::with_status(
                StatusCode::BAD_REQUEST,
                "terminal receipt is malformed, oversized, or contains raw secret material",
            )
        })?;
        if append_target_operation_snapshot(
            state.runtime.store().as_ref(),
            state.target_agents.as_ref(),
            state.target_agents.operation_next_sequence(),
            &next,
        )
        .await
        .map_err(target_internal_error)?
        .is_some()
        {
            return Ok(Json(next));
        }
    }
    Err(ServiceError::with_status(
        StatusCode::CONFLICT,
        "target operation journal changed too frequently",
    ))
}

fn validate_agent_operation_binding(
    operation: &TargetOperationRecord,
    agent: &StoredAgent,
    request_digest: &str,
    authority_digest: &str,
    execution_id: Option<&str>,
) -> Result<(), ServiceError> {
    if operation.authority.lease_epoch != agent.target.lease_epoch
        || operation.authority.policy_epoch != agent.target.policy_epoch
        || operation.authority.request_digest != request_digest
        || operation.authority.authority_digest != authority_digest
        || execution_id.is_some_and(|candidate| {
            !is_execution_id(candidate)
                || operation
                    .execution_id
                    .as_deref()
                    .is_some_and(|expected| expected != candidate)
        })
    {
        return Err(ServiceError::with_status(
            StatusCode::CONFLICT,
            "operation authority, request digest, or target epoch did not match",
        ));
    }
    Ok(())
}

async fn stream_operation_artifact<S>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    Path((operation_id, digest)): Path<(String, String)>,
) -> Result<Response, ServiceError>
where
    S: EventStore,
{
    validate_sha256_digest(&digest)?;
    let agent = authenticated_agent(&state, &headers).await?;
    sync_target_operation_journal(state.runtime.store().as_ref(), state.target_agents.as_ref())
        .await
        .map_err(target_internal_error)?;
    let operation = state
        .target_agents
        .operation(&operation_id)
        .filter(|operation| operation.target_id == agent.target.id)
        .ok_or_else(|| ServiceError::with_status(StatusCode::NOT_FOUND, "operation not found"))?;
    validate_agent_operation_binding(
        &operation,
        &agent,
        &operation.authority.request_digest,
        &operation.authority.authority_digest,
        None,
    )?;
    if !matches!(
        operation.status,
        TargetOperationStatusKind::Accepted | TargetOperationStatusKind::Running
    ) || !operation.authority.artifact_digests.contains(&digest)
    {
        return Err(ServiceError::with_status(
            StatusCode::FORBIDDEN,
            "artifact is not authorized for this accepted operation",
        ));
    }

    let object_store = state.runtime.object_store();
    let info = object_store
        .verify(&digest)
        .await
        .map_err(|error| match error {
            ygg_runtime::ObjectStoreError::NotFound { .. } => ServiceError::with_status(
                StatusCode::NOT_FOUND,
                "authorized artifact was not found",
            ),
            other => target_internal_error(other.into()),
        })?;
    if operation
        .spec
        .expected_size(&digest)
        .is_some_and(|expected| expected != info.size_bytes)
    {
        return Err(ServiceError::with_status(
            StatusCode::CONFLICT,
            "authorized artifact size does not match the operation descriptor",
        ));
    }
    let reader = object_store
        .stream(&digest)
        .await
        .map_err(|error| target_internal_error(error.into()))?;
    let body_stream = futures::stream::try_unfold(reader, |mut reader| async move {
        let mut chunk = vec![0u8; 64 * 1024];
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            Ok::<_, std::io::Error>(None)
        } else {
            chunk.truncate(read);
            Ok::<_, std::io::Error>(Some((chunk, reader)))
        }
    });
    let etag = format!("\"{}\"", info.digest);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, info.size_bytes)
        .header(header::ETAG, etag)
        .header("x-ygg-artifact-digest", info.digest)
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from_stream(body_stream))
        .map_err(|error| target_internal_error(error.into()))
}

async fn create_operation_record<S>(
    store: &S,
    registry: &TargetAgentRegistry,
    authority_key: &str,
    target: ExecutionTarget,
    request: CreateTargetOperationRequest,
) -> anyhow::Result<TargetOperationRecord>
where
    S: EventStore,
{
    let request_digest = operation_request_digest(&target.id, &request.project_id, &request.spec)?;
    let operation_id = ygg_core::new_id("target-operation");
    let now_ms = Utc::now().timestamp_millis();
    let expires_at_ms = now_ms.saturating_add(
        i64::try_from(
            request
                .expires_in_seconds
                .unwrap_or(DEFAULT_AUTHORITY_TTL_SECS),
        )
        .unwrap_or(i64::MAX)
        .saturating_mul(1_000),
    );
    let mut authority = TargetOperationAuthority {
        target_id: target.id.clone(),
        operation_id: operation_id.clone(),
        step_id: OPERATION_STEP_ID.to_string(),
        project_id: request.project_id.clone(),
        effect: request.spec.effect(),
        artifact_digests: request.spec.artifact_digests(),
        lease_epoch: target.lease_epoch,
        policy_epoch: target.policy_epoch,
        issued_at_ms: now_ms,
        expires_at_ms,
        nonce: random_secret(),
        request_digest,
        authority_digest: String::new(),
    };
    authority.authority_digest = operation_authority_digest(&authority, authority_key)?;
    let record = TargetOperationRecord {
        operation_id,
        target_id: target.id,
        project_id: request.project_id,
        revision: 1,
        status: TargetOperationStatusKind::Requested,
        execution_id: None,
        spec: request.spec,
        authority,
        idempotency_key: request.idempotency_key,
        receipt: None,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    validate_record_integrity(&record, authority_key)?;

    for _ in 0..8 {
        sync_target_operation_journal(store, registry).await?;
        if let Some(key) = record.idempotency_key.as_deref() {
            if let Some((existing_digest, existing)) =
                registry.idempotent_operation(&record.target_id, &record.project_id, key)
            {
                anyhow::ensure!(
                    existing_digest == record.authority.request_digest,
                    "idempotency_key was already used for a different target operation"
                );
                return Ok(existing);
            }
        }
        if append_target_operation_snapshot(
            store,
            registry,
            registry.operation_next_sequence(),
            &record,
        )
        .await?
        .is_some()
        {
            return Ok(record);
        }
    }
    anyhow::bail!("target operation journal changed too frequently to create operation")
}

async fn append_target_operation_snapshot<S>(
    store: &S,
    registry: &TargetAgentRegistry,
    expected_next: EventSequence,
    record: &TargetOperationRecord,
) -> anyhow::Result<Option<EventEnvelope>>
where
    S: EventStore,
{
    let authority_key = registry
        .operation_authority_key(&record.target_id, record.authority.lease_epoch)
        .ok_or_else(|| anyhow::anyhow!("target operation authority key is unknown"))?;
    validate_record_integrity(record, &authority_key)?;
    let event = store
        .append_with_sequence_if_next(
            OPERATION_JOURNAL_SESSION.to_string(),
            expected_next,
            JOURNAL_WRITER.to_string(),
            OPERATION_JOURNAL_EVENT.to_string(),
            1,
            serde_json::to_value(TargetOperationSnapshot {
                record: record.clone(),
            })?,
            json!({
                "owner": "host_control_plane",
                "redacted": true,
                "authority_material": "digest_bound"
            }),
        )
        .await?;
    if let Some(event) = &event {
        registry.apply_operation_event(event)?;
    }
    Ok(event)
}

pub(super) async fn sync_target_operation_journal<S>(
    store: &S,
    registry: &TargetAgentRegistry,
) -> anyhow::Result<usize>
where
    S: EventStore,
{
    let mut loaded = 0usize;
    loop {
        let next = registry.operation_next_sequence();
        let events = store
            .list_session_range(
                &OPERATION_JOURNAL_SESSION.to_string(),
                next.checked_sub(1),
                Some(1_000),
            )
            .await?;
        if events.is_empty() {
            break;
        }
        for event in &events {
            registry.apply_operation_event(event)?;
            loaded = loaded.saturating_add(1);
        }
        if events.len() < 1_000 {
            break;
        }
    }
    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ygg_runtime::InMemoryEventStore;

    const TEST_CREDENTIAL: &str =
        "yggagent.remote-1.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn test_registry() -> TargetAgentRegistry {
        let registry = TargetAgentRegistry::default();
        registry.state.lock().unwrap().credential_digests.insert(
            ("remote-1".to_string(), 7),
            credential_digest("agent", TEST_CREDENTIAL),
        );
        registry
    }

    fn test_target() -> ExecutionTarget {
        ExecutionTarget {
            id: "remote-1".to_string(),
            name: "remote verifier".to_string(),
            reachability: ExecutionTargetReachability::ReverseTunnel,
            declared_capabilities: vec![
                ExecutionTargetCapability::ArtifactTransfer,
                ExecutionTargetCapability::DeclarativeVerifier,
            ],
            capabilities: vec![
                ExecutionTargetCapability::ArtifactTransfer,
                ExecutionTargetCapability::DeclarativeVerifier,
            ],
            status: ExecutionTargetStatusKind::Available,
            protocol_versions: vec![PROTOCOL_VERSION.to_string()],
            selected_protocol_version: Some(PROTOCOL_VERSION.to_string()),
            identity_ref: Some("target:remote-1:7".to_string()),
            labels: BTreeMap::new(),
            observed: Some(ExecutionTargetObservedSummary::default()),
            last_seen_at_ms: Some(Utc::now().timestamp_millis()),
            heartbeat_expires_at_ms: Some(Utc::now().timestamp_millis() + HEARTBEAT_TTL_MS),
            enrolled_at_ms: Some(Utc::now().timestamp_millis()),
            revoked_at_ms: None,
            lease_epoch: 7,
            policy_epoch: 11,
        }
    }

    fn operation_request(key: &str) -> CreateTargetOperationRequest {
        CreateTargetOperationRequest {
            project_id: ProjectId::new("project-1").unwrap(),
            spec: TargetOperationSpec::VerifierRun {
                verifier: DeclarativeVerifierDescriptor::ArtifactIntegrity {
                    digest: format!("sha256:{}", "a".repeat(64)),
                    expected_size_bytes: Some(42),
                },
            },
            idempotency_key: Some(key.to_string()),
            expires_in_seconds: Some(120),
        }
    }

    #[tokio::test]
    async fn operation_idempotency_and_hydration_preserve_one_authority() -> anyhow::Result<()> {
        let store = InMemoryEventStore::default();
        let registry = test_registry();
        let authority_key = credential_digest("agent", TEST_CREDENTIAL);
        let first = create_operation_record(
            &store,
            &registry,
            &authority_key,
            test_target(),
            operation_request("retry-1"),
        )
        .await?;
        let duplicate = create_operation_record(
            &store,
            &registry,
            &authority_key,
            test_target(),
            operation_request("retry-1"),
        )
        .await?;
        assert_eq!(first.operation_id, duplicate.operation_id);
        assert_eq!(first.authority, duplicate.authority);

        let restored = test_registry();
        assert_eq!(sync_target_operation_journal(&store, &restored).await?, 1);
        assert_eq!(restored.operation(&first.operation_id), Some(first));
        Ok(())
    }

    #[tokio::test]
    async fn authority_tampering_and_raw_secret_receipts_fail_closed() -> anyhow::Result<()> {
        let store = InMemoryEventStore::default();
        let registry = test_registry();
        let authority_key = credential_digest("agent", TEST_CREDENTIAL);
        let record = create_operation_record(
            &store,
            &registry,
            &authority_key,
            test_target(),
            operation_request("retry-1"),
        )
        .await?;
        verify_target_operation_authority(
            &record,
            TEST_CREDENTIAL,
            "remote-1",
            7,
            11,
            record.created_at_ms,
            true,
        )
        .map_err(anyhow::Error::msg)?;
        assert!(verify_target_operation_authority(
            &record,
            "yggagent.remote-1.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "remote-1",
            7,
            11,
            record.created_at_ms,
            true,
        )
        .is_err());

        let mut tampered = record.clone();
        tampered.authority.artifact_digests.clear();
        assert!(verify_target_operation_authority(
            &tampered,
            TEST_CREDENTIAL,
            "remote-1",
            7,
            11,
            tampered.created_at_ms,
            true,
        )
        .is_err());

        let mut completed = record;
        completed.revision = 2;
        completed.status = TargetOperationStatusKind::Succeeded;
        completed.execution_id = Some("b".repeat(32));
        completed.receipt = Some(TargetOperationReceipt {
            operation_id: completed.operation_id.clone(),
            target_id: completed.target_id.clone(),
            execution_id: "b".repeat(32),
            step_id: OPERATION_STEP_ID.to_string(),
            request_digest: completed.authority.request_digest.clone(),
            authority_digest: completed.authority.authority_digest.clone(),
            status: TargetOperationReceiptStatus::Succeeded,
            completed_at_ms: completed.created_at_ms,
            output: json!({ "access_token": "sk-abcdefghijklmnopqrstuvwxyz123456" }),
            diagnostics: Vec::new(),
        });
        assert!(validate_record_integrity(&completed, &authority_key).is_err());
        Ok(())
    }
}
