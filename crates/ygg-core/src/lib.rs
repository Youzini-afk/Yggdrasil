pub mod asset;
pub mod capability_handle;
pub mod change;
pub mod conformance;
pub mod effect;
pub mod event;
pub mod ids;
pub mod lockfile;
pub mod manifest;
pub mod paths;
pub mod project;
pub mod secret_ref;
pub mod session;

pub use asset::{ArtifactDescriptor, AssetRecord};
pub use capability_handle::{CapHandle, CapHandleId, HandleLease, HandleProvenance, HandleScope};
pub use change::{
    ChangeCommit, ChangeCommitStatus, ChangeOperation, ChangePrecondition, ChangeSet, Intent,
    PolicyDecision, PolicyDecisionOutcome, CHANGE_COMMIT_TYPE_URI, CHANGE_SET_TYPE_URI,
    INTENT_TYPE_URI,
};
pub use effect::{
    EffectReceipt, EffectReplayMode, EffectScope, EffectTerminalStatus, PrincipalIdentity,
    APPROVAL_EVIDENCE_TYPE_URI, AUTHORITY_EVIDENCE_TYPE_URI, COMPONENT_EVIDENCE_TYPE_URI,
    EFFECT_RECEIPT_TYPE_URI, EFFECT_VALUE_TYPE_URI, POLICY_DECISION_TYPE_URI,
};
pub use event::{
    EventEnvelope, EventKind, EventSequence, OutboundAuditRecord, PackageLifecyclePayload,
    RedactionState, SchemaVersion, StreamFrameEnvelope, StreamFrameType, StreamInvocationRecord,
    StreamInvocationState, EVENT_ASSET_PUT, EVENT_CAPABILITY_COMPLETED, EVENT_CAPABILITY_FAILED,
    EVENT_CAPABILITY_INVOKED, EVENT_DEPLOYMENT_HEALTH, EVENT_DEPLOYMENT_RECONCILED, EVENT_ERROR,
    EVENT_EXEC_COMPLETED, EVENT_EXEC_DENIED, EVENT_EXEC_FAILED, EVENT_EXEC_REQUEST,
    EVENT_EXEC_STARTED, EVENT_EXEC_STOPPED, EVENT_OUTBOUND_DENIED,
    EVENT_OUTBOUND_EXECUTE_COMPLETED, EVENT_OUTBOUND_REQUEST, EVENT_OUTBOUND_STREAM_COMPLETED,
    EVENT_OUTBOUND_WEBSOCKET_COMPLETED, EVENT_OUTBOUND_WEBSOCKET_ERROR,
    EVENT_OUTBOUND_WEBSOCKET_FRAME, EVENT_OUTBOUND_WEBSOCKET_OPENED, EVENT_PACKAGE_DEGRADED,
    EVENT_PACKAGE_LOADED, EVENT_PACKAGE_LOADING, EVENT_PACKAGE_LOG, EVENT_PACKAGE_READY,
    EVENT_PACKAGE_STARTING, EVENT_PACKAGE_STOPPED, EVENT_PACKAGE_STOPPING, EVENT_PACKAGE_UNLOADED,
    EVENT_PERMISSION_DENIED, EVENT_PERMISSION_GRANTED, EVENT_PERMISSION_REVOKED, EVENT_PORT_DENIED,
    EVENT_PORT_LEASED, EVENT_PORT_RELEASED, EVENT_PROJECTION_UPDATED, EVENT_PROPOSAL_APPLIED,
    EVENT_PROPOSAL_APPROVED, EVENT_PROPOSAL_CREATED, EVENT_PROPOSAL_FAILED,
    EVENT_PROPOSAL_REJECTED, EVENT_PROXY_DENIED, EVENT_PROXY_REGISTERED, EVENT_PROXY_UNREGISTERED,
    EVENT_SESSION_CLOSED, EVENT_SESSION_FORKED, EVENT_SESSION_OPENED, EVENT_STREAM_CANCELLED,
    EVENT_STREAM_CHUNK, EVENT_STREAM_ENDED, EVENT_STREAM_ERROR, EVENT_STREAM_PROGRESS,
    EVENT_STREAM_STARTED, EVENT_STREAM_TIMEOUT, KERNEL_PACKAGE_ID, PROJECT_INSTALLED,
    PROJECT_STARTED, PROJECT_STOPPED, PROJECT_UNINSTALLED,
};
pub use ids::{
    new_id, AssetId, CapabilityId, EventId, ExtensionPointId, HookId, InvocationId, PackageId,
    PrincipalId, SessionId,
};
pub use lockfile::{LockEntry, LockRequirement, LockSource, Lockfile};
pub use manifest::{
    AssetPermissions, CapabilityDescriptor, CapabilityPermissions, CapabilityRequirement,
    ContractMode, DependencySource, EntryDescriptor, EventPermissions, ExtensionPointDescriptor,
    FilesystemPermissions, HookSubscription, HookTiming, LocalExecDeclaration,
    LocalExecPermissions, ManifestError, NetworkDeclaration, NetworkPermissions,
    PackageContributions, PackageDependency, PackageEntry, PackageManifest, PackagePermissions,
    PermissionSet, PortDeclaration, PortPermissions, ProxyDeclaration, ProxyPermissions,
    RemoteAuth, SandboxPolicy, SchemaContribution, SubprocessTransport, SurfaceActivation,
    SurfaceApprovalPolicy, SurfaceContribution, SurfacePermissionRequirement, SurfaceRisk,
    SurfaceSlot,
};
pub use paths::{
    archived_project_dir, archived_projects_dir, cache_dir, data_dir, ensure_initialized,
    ensure_project_initialized, keys_dir, lockfile_path, profile_path, profiles_dir,
    project_descriptor_path, project_dir, project_lockfile_path, project_secret_store_path,
    project_sessions_dir, project_state_dir, project_workspace_dir, project_workspace_dir_in,
    projects_dir, secret_store_key_path, secret_store_path, store_dir, store_path_for_hash,
};
pub use project::{
    ExternalProjectData, ProjectDescriptor, ProjectId, ProjectInner, ProjectState, ProjectType,
    SecretPolicy,
};
pub use secret_ref::{
    extract_project_name, extract_store_name, is_env_backed_ref, is_project_backed_ref,
    is_secret_field_name, is_store_backed_ref, looks_like_raw_secret, SecretRef,
    SECRET_FIELD_NAMES, SECRET_REF_PREFIX,
};
pub use session::{KernelSession, SessionStatus};
