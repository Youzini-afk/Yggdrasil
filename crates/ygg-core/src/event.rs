use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::ids::{CapabilityId, EventId, PackageId, SessionId};

pub type SchemaVersion = u16;
pub type EventKind = String;
pub type EventSequence = u64;

pub const KERNEL_PACKAGE_ID: &str = "kernel";
pub const EVENT_SESSION_OPENED: &str = "kernel/session.opened";
pub const EVENT_SESSION_CLOSED: &str = "kernel/session.closed";
pub const EVENT_SESSION_FORKED: &str = "kernel/session.forked";
pub const EVENT_PACKAGE_LOADED: &str = "kernel/package.loaded";
pub const EVENT_PACKAGE_LOADING: &str = "kernel/package.loading";
pub const EVENT_PACKAGE_STARTING: &str = "kernel/package.starting";
pub const EVENT_PACKAGE_READY: &str = "kernel/package.ready";
pub const EVENT_PACKAGE_STOPPING: &str = "kernel/package.stopping";
pub const EVENT_PACKAGE_STOPPED: &str = "kernel/package.stopped";
pub const EVENT_PACKAGE_UNLOADED: &str = "kernel/package.unloaded";
pub const EVENT_PACKAGE_DEGRADED: &str = "kernel/package.degraded";
pub const EVENT_PACKAGE_LOG: &str = "kernel/package.log";
pub const EVENT_ASSET_PUT: &str = "kernel/asset.put";
pub const EVENT_PROJECTION_UPDATED: &str = "kernel/projection.updated";
pub const EVENT_PROPOSAL_CREATED: &str = "kernel/proposal.created";
pub const EVENT_PROPOSAL_APPROVED: &str = "kernel/proposal.approved";
pub const EVENT_PROPOSAL_REJECTED: &str = "kernel/proposal.rejected";
pub const EVENT_PROPOSAL_APPLIED: &str = "kernel/proposal.applied";
pub const EVENT_PROPOSAL_FAILED: &str = "kernel/proposal.failed";
pub const EVENT_CAPABILITY_INVOKED: &str = "kernel/capability.invoked";
pub const EVENT_CAPABILITY_COMPLETED: &str = "kernel/capability.completed";
pub const EVENT_CAPABILITY_FAILED: &str = "kernel/capability.failed";
pub const EVENT_PERMISSION_DENIED: &str = "kernel/permission.denied";
pub const EVENT_PERMISSION_GRANTED: &str = "kernel/permission.granted";
pub const EVENT_PERMISSION_REVOKED: &str = "kernel/permission.revoked";
pub const EVENT_ERROR: &str = "kernel/error";
pub const EVENT_OUTBOUND_REQUEST: &str = "kernel/outbound.request";
pub const EVENT_OUTBOUND_DENIED: &str = "kernel/outbound.denied";

// ---------------------------------------------------------------------------
// Outbound audit / redaction types (Phase S2)
// ---------------------------------------------------------------------------

/// Redaction state for an outbound audit record.
///
/// Every outbound request carries one of these states to indicate
/// whether raw body/header/prompt/response data was preserved.
/// The default is `NotCaptured` — raw data is never saved unless
/// explicitly approved.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RedactionState {
    /// Raw data was not captured (default).
    NotCaptured,
    /// Raw data was redacted before recording.
    Redacted,
    /// Only a policy reference is stored; no data captured.
    PolicyRef,
    /// Request was blocked as unsafe; no data recorded.
    UnsafeBlocked,
    /// Explicit user/host approval to capture raw data (rare).
    ExplicitlyApproved,
}

impl Default for RedactionState {
    fn default() -> Self {
        Self::NotCaptured
    }
}

/// Generic outbound audit record / envelope.
///
/// Records an outbound network request made by a package through
/// Ygg-provided network/request helpers. This is a kernel event
/// payload — it does NOT contain raw secrets, bodies, headers,
/// prompts, or responses. Only `secret_ref` identifiers and the
/// `redaction_state` are recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundAuditRecord {
    /// Unique record id.
    pub id: String,
    /// The principal that initiated the request.
    pub principal: String,
    /// Package that owns the outbound request.
    pub package_id: PackageId,
    /// Capability through which the request was made.
    pub capability_id: CapabilityId,
    /// Destination host.
    pub destination_host: String,
    /// HTTP method (GET, POST, etc).
    pub method: String,
    /// Declared purpose from the manifest or request context.
    #[serde(default)]
    pub purpose: Option<String>,
    /// Redaction state — what data, if any, was recorded.
    #[serde(default)]
    pub redaction_state: RedactionState,
    /// Secret references used (not raw secrets).
    #[serde(default)]
    pub secret_refs_used: Vec<String>,
    /// Usage placeholder (e.g. token count).
    #[serde(default)]
    pub usage: Value,
    /// Cost placeholder.
    #[serde(default)]
    pub cost: Value,
    /// Request status: "allowed", "denied", "error", etc.
    pub status: String,
    /// Error message if status is not "allowed".
    #[serde(default)]
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Type aliases and EventEnvelope
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: EventId,
    pub session_id: SessionId,
    pub sequence: EventSequence,
    pub writer_package_id: PackageId,
    pub kind: EventKind,
    pub schema_version: SchemaVersion,
    pub timestamp: DateTime<Utc>,
    pub payload: Value,
    #[serde(default)]
    pub metadata: Value,
}

impl EventEnvelope {
    pub fn new(
        id: EventId,
        session_id: SessionId,
        sequence: EventSequence,
        writer_package_id: PackageId,
        kind: impl Into<EventKind>,
        payload: Value,
    ) -> Self {
        Self {
            id,
            session_id,
            sequence,
            writer_package_id,
            kind: kind.into(),
            schema_version: 1,
            timestamp: Utc::now(),
            payload,
            metadata: Value::Object(Map::new()),
        }
    }

    pub fn is_kernel_event(&self) -> bool {
        self.writer_package_id == KERNEL_PACKAGE_ID && self.kind.starts_with("kernel/")
    }

    pub fn writer_owns_kind(&self) -> bool {
        if self.kind.starts_with("kernel/") {
            return self.writer_package_id == KERNEL_PACKAGE_ID;
        }
        self.kind.starts_with(&format!("{}/", self.writer_package_id))
    }
}
