pub mod capability;
pub mod event_store;
pub mod inproc;
pub mod package;
pub mod pi;
pub mod protocol;
pub mod redaction;
pub mod runtime;
pub mod schema;
pub mod secret;
pub mod storage;
pub mod subprocess;
pub mod tavern;

pub use capability::{
    CapabilityFabric, CapabilityInvocationRequest, CapabilityInvocationResult, ExtensionDispatchResult,
    ExtensionRegistry, RegisteredCapability, RegisteredHook,
};
pub use event_store::{EventStore, InMemoryEventStore, SqliteEventStore};
pub use inproc::{InprocInvocation, InprocPackage, InprocPackageCatalog};
pub use package::{entry_kind, trust_level, HostPolicy, PackageRecord, PackageRegistry, PackageState, TrustLevel};
pub use pi::PI_INTEGRATION_DEFERRED;
pub use protocol::{
    host_info, method_ids, HostInfo, KernelMethod, MethodStatus, ProtocolContext, ProtocolError,
    ProtocolMethod, ProtocolPrincipal, ProtocolRequest, ProtocolResponse, KERNEL_METHODS,
    KERNEL_PROTOCOL_VERSION,
};
pub use redaction::{redact_secrets_in_value, scan_value_for_raw_secrets, SecretDetection, SecretFinding, SecretScanResult};
pub use runtime::{
    AppendEventRequest, DenyAllOutboundExecutor, EventListRequest, ExecutorKind,
    FakeOutboundExecutor, LiveHttpOutboundExecutor, LiveHttpOutboundExecutorConfig,
    NetworkPolicyDecision, OpenSessionRequest, OutboundExecutor, OutboundExecutorConfig,
    OutboundExecutorRequest, OutboundExecutorResponse, OutboundRequest, Runtime, RuntimeConfig,
    StreamRegistry, check_network_policy,
};
pub use schema::validate_json_schema_subset;
pub use secret::{DenyAllSecretResolver, EnvSecretResolver, HostSecretResolver, SecretResolverConfig, extract_env_name};
pub use subprocess::{SubprocessLogLine, SubprocessSupervisor};
pub use tavern::TAVERN_COMPAT_DEFERRED;
