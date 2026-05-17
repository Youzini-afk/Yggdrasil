use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::ids::{EventId, PackageId, SessionId};

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
