# Yggdrasil Outbound Evolution Plan (YdlTavern Co-evolution)

> [English](./YGGDRASIL_OUTBOUND_EVOLUTION_PLAN.en.md) · [中文](./YGGDRASIL_OUTBOUND_EVOLUTION_PLAN.md)
>
> Temporary document. Yggdrasil-side co-evolution plan aligned with YdlTavern's push. Update after each phase; delete entirely and merge into long-term docs (`docs/protocol/PROTOCOL_V0.md`, `docs/guides/MODEL_PROVIDER_INTEGRATION.md`, `docs/ALPHA_STATUS.md`, etc.) once all phases complete.

## Positioning

YdlTavern is Yggdrasil's first real user. Building YdlTavern exposed several actual gaps in Yggdrasil:

```text
Gap 1: HostProfile has no outbound.execute section (only outbound.git)
Gap 2: Package manifest has no secret_ref declaration field
Gap 3: kernel.outbound.execute is unary, no streaming (required for live model calls)
Gap 4: Subprocess JSON-RPC boundary has no kernel.outbound.* client helper
Gap 5: docs/protocol/PROTOCOL_V0.md does not document outbound methods
Gap 6: No forge profile template demonstrating live executor configuration
```

This round closes all these gaps at once. Only after closing them can Yggdrasil truly enable YdlTavern (and future external packages) to achieve "real model calls routed through the platform + secrets never stored directly in packages + streaming + audit".

## Design decisions (confirmed)

```text
- Streaming as a kernel capability: add kernel.outbound.stream public protocol method
- secret_refs declared in manifest: fail-closed rejection of undeclared secret references
- Live HTTP outbound remains opt-in: default deny_all, profile explicitly enables
- Subprocess calls kernel.outbound.* through SDK helper: no hand-written JSON-RPC on the YdlTavern side
- No kernel namespace pollution: no chat/turn/agent concepts added
- No special path for YdlTavern: all changes are general-purpose infrastructure
```

## Phase overview

| Phase | Content | Depends on |
|---|---|---|
| Y0 | Plan submission | - |
| Y1 | HostProfile.outbound.execute schema + executor builder | - |
| Y2 | manifest permissions.secret_refs declaration + validation | - |
| Y3 | kernel.outbound.stream protocol method + LiveHttpStreamingExecutor | Y1, Y2 |
| Y4 | Subprocess SDK kernel.outbound.* helper | Y3 |
| Y5 | forge-with-live-models.example.yaml + full conformance suite | Y1-Y4 |
| Y6 | Doc convergence (PROTOCOL_V0 / MODEL_PROVIDER_INTEGRATION / ALPHA_STATUS) | Y1-Y5 |

## Y1: HostProfile.outbound.execute schema

**Location**: `crates/ygg-cli/src/cli.rs`, `crates/ygg-cli/src/commands/host.rs`

### Changes

```text
HostProfile YAML new section:

outbound:
  git:                                  # existing
    enabled: true
    executor: real
    allowed_hosts: [...]
    ...
  execute:                              # new
    enabled: false                      # default false
    executor: deny_all                  # deny_all | fake | live
    allowed_hosts: []                   # exact host or *.wildcard
    https_only: true
    timeout_ms: 30000
    allow_redirects: false
    allow_insecure_loopback_for_tests: false

Rust side:
  HostProfile struct gains outbound.execute field
  build_outbound_execute_executor(config) -> Box<dyn OutboundExecutor>
    selects DenyAll / Fake / LiveHttp based on executor field
  Runtime::with_outbound_executor injected at host serve startup

LiveHttpOutboundExecutor::new_from_profile(config) new constructor
```

### Verification

```text
cargo check -p ygg-cli
cargo test -p ygg-cli (new host_profile_execute_*)
cargo test -p ygg-runtime (verify executor injection correct)
default forge-alpha.yaml does not configure execute (keeps deny_all default behavior)
```

## Y2: manifest permissions.secret_refs declaration

**Location**: `crates/ygg-core/src/manifest.rs`, `crates/ygg-runtime/src/runtime/protocol_dispatch.rs`

### Changes

```text
manifest.yaml new field:

permissions:
  network:
    declarations:
      - host: api.openai.com
        methods: [POST]
  secret_refs:                          # new
    - secret_ref:env:OPENAI_API_KEY
    - secret_ref:env:DEEPSEEK_API_KEY

Rust side:
  PermissionsManifest::secret_refs: Vec<String>
  Parse-time validation of each secret_ref form (reuse secret.rs::parse_secret_ref)

dispatch_outbound_execute / _stream:
  For each secret_ref in secret_headers:
    Check whether it is in the caller package manifest.permissions.secret_refs list
    Undeclared → ProtocolError code=permission_denied, fail-closed

ygg conformance:
  outbound_secret_ref_undeclared_fails
  outbound_secret_ref_declared_resolves
```

### Boundaries

```text
Existing packages/official/* do not use secret_ref → no manifest changes needed
Backward compatible: missing secret_refs field treated as empty array
```

### Verification

```text
cargo test -p ygg-core (manifest parsing + validation)
cargo test -p ygg-runtime (dispatch-side fail-closed)
ygg conformance new cases pass
```

## Y3: kernel.outbound.stream protocol method

**Location**: `crates/ygg-runtime/src/protocol.rs`, `crates/ygg-runtime/src/runtime/outbound.rs`, `crates/ygg-runtime/src/runtime/protocol_dispatch.rs`, `crates/ygg-runtime/src/runtime/streaming.rs`

### Design

```text
New public protocol method:
  kernel.outbound.stream
    params: same as kernel.outbound.execute, plus stream_options { buffer_size, frame_format }
    return: { stream_id: uuid }
    then frames pushed via kernel/stream.* events

Frame envelope (reuse existing kernel/stream.* events):
  kernel/stream.started   { stream_id, capability_id, executor_kind, redaction_state }
  kernel/stream.chunk     { stream_id, frame_index, chunk_shape, redaction_state }
                          chunk_shape is redacted — no raw secret
                          but raw_chunk frame provided to subscribed subprocess via transport (see below)
  kernel/stream.ended     { stream_id, status, usage, cost, redaction_state }
  kernel/stream.error     { stream_id, code, message }
  kernel/stream.cancelled { stream_id, reason }
  kernel/stream.timeout   { stream_id, timeout_ms }

OutboundExecutor trait extension:
  fn execute_stream(&self, request: OutboundExecutorRequest, sink: StreamSink) -> Result<(), OutboundError>
    DenyAllOutboundExecutor: reject
    FakeOutboundExecutor: emit deterministic chunks (for testing)
    LiveHttpOutboundExecutor: reqwest streaming + tokio channel forward

On raw chunks delivered to subprocess:
  Option A: push raw bytes directly via kernel/stream.chunk events (events allow chunk_shape to be redacted text or base64)
  Option B: separate raw stream channel (subprocess SDK subscribes)
  Recommended: Option A — simple, reuses existing stream subsystem, redaction policy applied on chunk_shape
  For SSE text streams, chunk_shape contains decoded SSE lines (with secret patterns redacted)

Cancel/timeout:
  kernel.capability.cancel accepts stream_id (already exists)
  per-stream timeout triggers stream.timeout + emit ended
```

### Changes

```text
crates/ygg-runtime/src/protocol.rs
  KernelMethod::OutboundStream added
  Parameter schema registered in ProtocolRegistry

crates/ygg-runtime/src/runtime/protocol_dispatch.rs
  fn dispatch_outbound_stream
  Permission checks same as outbound_execute
  secret_ref declaration check (Y2)
  Allow streaming executor (reject deny_all)

crates/ygg-runtime/src/runtime/outbound.rs
  trait OutboundExecutor::execute_stream (default unsupported)
  StreamSink trait
  LiveHttpStreamingExecutor implementation (reqwest::Response::bytes_stream)
  FakeStreamingExecutor implementation

crates/ygg-runtime/src/runtime/streaming.rs
  Outbound stream integrated into capability stream lifecycle
  stream_id reuses same uuid namespace
  redaction policy applied to chunk_shape

docs:
  PROTOCOL_V0 new outbound.stream section
  MODEL_PROVIDER_INTEGRATION new streaming section
```

### Verification

```text
cargo test -p ygg-runtime (outbound_stream_* unit tests)
ygg conformance:
  outbound_stream_lifecycle_started_chunk_ended
  outbound_stream_cancel_emits_cancelled
  outbound_stream_timeout_emits_timeout
  outbound_stream_fake_emits_deterministic_chunks
  outbound_stream_secret_ref_undeclared_fails
  outbound_stream_executor_kind_audited
```

## Y4: Subprocess SDK kernel.outbound.* helper

**Location**: `sdk/typescript/subprocess/`, `crates/ygg-runtime/src/subprocess.rs`

### Design

```text
Current state:
  subprocess SDK can only respond to capability.invoke
  No path to initiate kernel.outbound.* calls

Target:
  subprocess SDK provides kernelClient
    sendKernelRequest<T>(method: string, params: unknown): Promise<T>
    streamKernelRequest(method: string, params: unknown, callbacks: { onChunk, onEnd, onError, onCancelled, onTimeout }): { cancel: () => void }

  Internal implementation:
    Emit JSON-RPC envelope via stdout (method is kernel.* not capability.invoke)
    Runtime side subprocess.rs recognizes this as a reverse call initiated by subprocess
    Forward to ProtocolRegistry::dispatch (with caller principal = subprocess package)
    Response sent back via stdin (with matching id)
    Streaming chunks pushed via stdin (kernel/stream.chunk events)

Boundaries:
  caller principal strictly bound to subprocess's package_id
  Spoofing another package is not allowed
  permission/secret_ref checks same as public protocol path
```

### Changes

```text
sdk/typescript/subprocess/src/
  kernel-client.ts (new)
    KernelClient class
    sendKernelRequest / streamKernelRequest
    Internal id generation, pending Promise management, stdin parsing

  outbound.ts (new)
    executeOutbound(params): Promise<OutboundResponse>
    streamOutbound(params, callbacks): { cancel }
    Wraps kernel-client with typed input/output

  index.ts exports KernelClient / outbound helpers

crates/ygg-runtime/src/subprocess.rs
  Parse subprocess stdout JSON-RPC:
    method == 'capability.invoke' → existing logic
    method.starts_with('kernel.') → forward to ProtocolRegistry::dispatch
    Response written back to stdin, with caller principal=subprocess package
    Streaming events written back to stdin (kernel/stream.* envelopes)
```

### Verification

```text
sdk/typescript/subprocess typecheck/build
New examples/packages/subprocess-outbound-canary/
  manifest declares network + secret_refs (using fake env var)
  capability calls kernel.outbound.execute via SDK (FakeExecutor path)
  capability calls kernel.outbound.stream via SDK (verify chunks received)
  cancel path verification

cargo test -p ygg-runtime subprocess_outbound_*
ygg conformance subprocess_outbound_through_kernel_*
```

## Y5: forge-with-live-models.example.yaml + conformance

**Location**: `profiles/`, `crates/ygg-cli/src/conformance/`

### Changes

```text
profiles/forge-with-live-models.example.yaml (new):
  States this is an example, not a default profile
  Complete outbound.execute live configuration
  Comment guidance: to use for real, must set OPENAI_API_KEY etc. env vars
                   + YGG_LIVE_MODEL_TESTS=1 to run live smoke tests

ygg conformance new classes:
  outbound_execute_*
  outbound_stream_*
  manifest_secret_refs_*
  subprocess_outbound_*

Expected total case count: current 329 + ~12 → ~341
```

### Verification

```text
cargo test --workspace
ygg conformance (all cases pass, including default still deny_all)
forge-with-live-models.example.yaml passes schema validation
Real live smoke test marked as opt-in (YGG_LIVE_MODEL_TESTS=1 + env vars)
Default CI still no network access
```

## Y6: Doc convergence

```text
docs/protocol/PROTOCOL_V0.md / .en.md
  outbound section: execute / stream / git_fetch
  Complete envelope examples
  Error code table
  redaction state enumeration

docs/guides/MODEL_PROVIDER_INTEGRATION.md / .en.md
  Host profile configuration example
  Manifest declaration example (network + secret_refs)
  Subprocess SDK usage example
  Streaming section
  YGG_LIVE_MODEL_TESTS opt-in mode

docs/ALPHA_STATUS.md / .en.md
  outbound stream + manifest secret_refs added to completion checklist

CONFORMANCE_MATRIX update
NEXT_STEPS remove completed items
```

## Boundaries (cross-cutting)

```text
No kernel content concepts added (chat/turn/agent etc. do not enter kernel)
No shortcuts for YdlTavern — all changes are general-purpose
Default profile does not allow network — opt-in is strict
No raw secret storage — all secret_ref + host resolver
No bypass of audit / redaction
No backward-incompatible manifest changes (missing secret_refs treated as empty array)
No unwrap()/panic!() — errors return Result
```

## Completion criteria

```text
Per phase:
  cargo check / test pass
  ygg conformance new cases all pass
  Existing cases 0 regression
  commit + push origin/main

Total completion:
  Y1-Y6 all pushed
  forge-with-live-models.example.yaml loadable
  Default forge-alpha.yaml behavior unchanged
  Docs in sync
  YdlTavern-side P3.5 can hook on
```

## Coupling points with YdlTavern plan

```text
YdlTavern P3.5 (model.live_call) depends on:
  Y1 (HostProfile.outbound.execute) - must come first
  Y2 (manifest.secret_refs)         - must come first
  Y3 (kernel.outbound.stream)       - streaming path required
  Y4 (subprocess SDK helper)        - YdlTavern engine call surface
  Y5 (example profile)              - user configuration reference

YdlTavern P1 (Golden Harness) has no dependency on Yggdrasil - fully parallel

YdlTavern P2 (Tokenizer) has no dependency on Yggdrasil - fully parallel
  But P2.5 validation depends on P1, still independent of Y*
```
