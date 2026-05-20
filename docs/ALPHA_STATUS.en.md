# Alpha Status

> [English](./ALPHA_STATUS.en.md) · [中文](./ALPHA_STATUS.md)

This is the living snapshot of what Yggdrasil is right now. It is updated whenever a milestone closes. It is not aspirational: every line below has code and conformance behind it (or is explicitly marked partial/deferred).

For the long-term architecture and product stance, see `docs/CHARTER.md`, `docs/architecture/VISION.md`, and `docs/product/PLAY_CREATION_MODEL.md`. For where this is going, see `docs/roadmap/NEXT_STEPS.md`.

## Headline

- **Stage:** Platform Foundation Alpha + Play/Forge Surface Contract Beta + Secure Execution Substrate Phase S1/S2/S3/S4 + Text Surface Proof Phase T1/T2/T3/T4/T5.
- **Conformance:** 150 named CLI cases plus crate and service unit tests.
- **Charter discipline:** kernel content-free, official packages no privilege, public protocol only, package equality across entry forms, raw-secret blocking in trusted paths, secret_ref references only, permission grants survive rehydrate, network permission enforcement with outbound audit/redaction, generic streaming and cancellation lifecycle, SDK secure-execution helpers, networked/streaming package templates, no-network readiness proof, **outbound executor boundary with deny-all default and fake executor conformance**.
- **Code health:** CLI commands/templates/conformance, runtime domain behavior, protocol dispatch, and runtime official in-process handlers are split by domain instead of accumulating in monolithic files.
- **Current headline:** Creative Inference Capability Alpha C3 delivered. C0 ADR, C1 transport-neutral inference capability contract, and C2 non-HTTP fake local provider proof are complete; C3 explicitly downgrades `official/model-provider-lab` into a cloud API adapter lab (not the Yggdrasil model abstraction, not an API gateway, no kernel privilege). Next stage is C4 Ygg-native inference proposal vertical slice.

## What is implemented

### Kernel

- Content-free sessions, append-only opaque events, manifest-driven packages, capability fabric, hook fabric slice, surface contributions, proposal lifecycle, asset/branch/projection substrate.
- SQLite-backed durable event log with per-session monotonic sequencing and rehydratable substrate including assets, branches, projections, and **permission grants**.
- JSON Schema subset for capability input/output and package-declared event payloads.
- Principals: `host_admin`, `host_dev`, `package`, `human`, `assistant`, `anonymous`. Scoped grants for human and assistant principals.
- **Persistent permission grants**: grants are rehydrated from the event log on runtime reconstruction. A grant issued before a host restart remains effective after rehydrate.
- Permission audit events: `kernel/permission.granted`, `kernel/permission.revoked`, `kernel/permission.denied`.
- **Secret reference contract**: `SecretRef` type with `secret_ref:<vault>:<key>`, `secretRef:`, `secret-ref:`, and `host:` reference patterns. Packages reference secrets via `secret_ref` identifiers; raw secrets must never appear in events, proposals, logs, or audit records.
- **Host secret resolver**: `HostSecretResolver` trait and `DenyAllSecretResolver` placeholder. Resolution is only permitted at runtime during capability invocation; resolved raw secrets must never be written back to the event log or any audit/proposal path. `SecretResolverConfig` on `RuntimeConfig`. **`EnvSecretResolver` (Phase L1)**: host-owned environment-variable resolver with explicit allowlist. Supports `secret_ref:env:NAME`, `secretRef:env:NAME`, `secret-ref:env:NAME`, `host:env:NAME`. Deny-all default; env names must be explicitly allowed. Missing/denied/malformed references return typed errors referencing env names but never raw values. `Runtime::resolve_secret_ref` host-internal method. Raw values exist only transiently; `Debug`/`Serialize`/audit never contain them. `host:<key>` references without `env:` prefix are not treated as env refs.
- **Raw-secret blocking**: Conservative scanner checks proposal payloads, asset metadata, and audit-like payloads for known secret field names (`api_key`, `secret`, `token`, `password`, etc.) and value patterns (`Bearer ...`, `sk-...`, high-entropy strings). Content/description/title/reason fields are excluded from value-pattern scanning to avoid false positives on ordinary text. Official packages have no bypass.
- **Network permission declarations**: Manifest `permissions.network` supports both flat `hosts` (backward compat) and structured `declarations` with `host`, `methods`, and `purpose`. The runtime policy checker matches outbound requests against declared entries. Packages with no network declarations are denied outbound access. Official packages have no bypass.
- **Outbound audit/redaction records**: `OutboundAuditRecord` records principal, package_id, capability_id, destination_host, method, purpose, redaction_state, secret_refs_used, usage/cost placeholders, status/error. Raw body/header/prompt/response is never saved — only `secret_ref` identifiers and the `redaction_state` enum (`not_captured`, `redacted`, `policy_ref`, `unsafe_blocked`, `explicitly_approved`) are recorded. Default is `redacted`.
    - **Network policy checker**: `check_network_policy` pure function and `check_and_audit_outbound` runtime method. Supports exact host match, wildcard prefix (`*.example.com`), method allowlists (empty = any), and flat `hosts` backward compat. Denied requests produce `kernel/outbound.denied` audit events; allowed requests produce `kernel/outbound.request` events.
- **Outbound executor boundary (M3 + L2 + L3 + L4 + L5)**: Content-free `OutboundExecutor` trait with `OutboundExecutorRequest` (package_id, capability_id, destination_host, method, path, purpose, secret_refs, redaction_state, timeout_ms, metadata, body_shape, **secret_headers**, **resolved_secret_headers**, **static_headers**) and `OutboundExecutorResponse` (status, status_code, headers_shape, body_shape, provider_request_id, usage, cost, redaction_state, network_performed, executor_kind). `DenyAllOutboundExecutor` returns denied without network (default, fail-closed). `FakeOutboundExecutor` provides deterministic fixtures by host/method/path with call counting for conformance, no real network. **`LiveHttpOutboundExecutor` (L2)**: Real HTTPS executor using reqwest + rustls. Disabled by default (`RuntimeConfig::default()` still uses `DenyAll`; requires explicit `OutboundExecutorConfig::LiveHttp`). HTTPS-only (rejects http:// URLs), redirect policy none (L2 does not implement redirect following), configurable timeout/connect_timeout, only sends `content-type: application/json` and `x-ygg-outbound` placeholder headers (no secret injection). **L4: `build_headers` injects resolved secret headers (e.g. Authorization) from `resolved_secret_headers`; raw values exist only in the HTTP request, never stored in audit/response/Debug.** Responses record only redacted `headers_shape` (auth/cookie etc. values as `[redacted]`), redacted `body_shape` (JSON secret fields replaced with `[redacted]`, non-JSON records kind/bytes_captured only), `provider_request_id` (from safe request-id headers only), `redaction_state` redacted, `network_performed` true, `executor_kind` Real. Errors normalized to `status="error"` or `"timeout"` with no raw body/secret. `allow_insecure_loopback_for_tests` flag defaults false, permits only `127.0.0.1`/`localhost` http:// URLs for conformance. `LiveHttpOutboundExecutorConfig` provides `timeout_ms`, `connect_timeout_ms`, `allow_redirects`, `max_response_preview_bytes`, `allow_insecure_loopback_for_tests`. `OutboundExecutorConfig` gains `LiveHttp(LiveHttpOutboundExecutorConfig)` variant; default remains `DenyAll`. `execute_outbound_with_policy` runtime method: first checks that policy/audit request and executor request agree on package/capability/host/method/secret_refs, then performs policy check; denied or inconsistent requests never call the executor; allowed requests call the executor; raw body is not persisted in audit; secret_refs stored as references only. Redirect-target following deferred to L4. **`kernel.outbound.execute` (L3 + L4)**: Public protocol method allowing ordinary packages to make outbound requests through the host outbound executor. Params accept capability_id, destination_host, method, path, secret_refs, metadata, body_shape. package_id is enforced from the ProtocolContext principal — callers cannot spoof a different package_id in params (host_dev/host_admin principals may specify package_id in params for testing). Dispatch calls `execute_outbound_with_policy`; response undergoes additional defense-in-depth raw-secret sweep (known secret field name values replaced with `[redacted]`). Does not add `kernel.secret.resolve` (raw secrets are never returned to packages). L3 does not inject secret headers (real injection deferred to L4/L5). `OutboundExecutorConfig` gains `LiveHttp(LiveHttpOutboundExecutorConfig)` variant; default remains `DenyAll`. `execute_outbound_with_policy` runtime method: first checks that policy/audit request and executor request agree on package/capability/host/method/secret_refs, then checks policy; denied or mismatched requests never call the executor, allowed requests do. Raw body is never in audit and secret_refs are references only. No provider-specific fields in core; opaque `metadata` for executor-specific data. This only secures the Ygg-provided outbound path and does not claim OS-level interception of arbitrary subprocess networking. Redirect-target following deferred to L4.
- **Protocol methods**: `kernel.outbound.audit` lists outbound audit events for a given package. `kernel.outbound.execute` allows ordinary packages to make outbound requests through the host outbound executor (L3).
- **Streaming invocation registry**: In-memory `StreamRegistry` tracks ongoing streaming capability invocations with start/append/end/cancel/timeout lifecycle. `StreamFrameEnvelope` defines generic content-free stream frame types (start/chunk/progress/end/error/cancelled/timeout) with invocation_id, stream_id, sequence, redaction_state, and timestamp/metadata. No model/prompt/agent semantics.
- **Streaming capability lifecycle**: `kernel.capability.stream` starts a streaming invocation (validates `streaming=true` in descriptor), `kernel.capability.cancel` cancels an in-flight invocation. Runtime methods emit ordered kernel events: `kernel/stream.started`, `kernel/stream.chunk`, `kernel/stream.progress`, `kernel/stream.ended`, `kernel/stream.error`, `kernel/stream.cancelled`, `kernel/stream.timeout`. Cancel and timeout block further chunks. Non-streaming capabilities (descriptor `streaming=false`) are rejected from streaming.
- **Streaming invocation record**: `StreamInvocationRecord` tracks invocation_id, stream_id, capability_id, provider_package_id, session_id, state (active/ended/error/cancelled/timeout), frame_count, timestamps, and metadata. Terminal states block further frame appends.
- **Secure-execution TypeScript helpers** (`sdk/typescript/secure-execution/index.ts`): `secretRef()` / `isValidSecretRef()` / `looksLikeRawSecret()` / `isSecretFieldName()` for secret reference construction and validation. `NetworkDeclaration` class for building manifest-compatible network permission entries with host/method matching. `OutboundAuditHelper` class for building audit-safe outbound request payloads that reject raw secrets and include only `secret_ref` identifiers. `StreamFrameClient` class for building faux stream frame envelopes with full lifecycle (start/chunk/progress/end/error/cancel/timeout). All helpers wrap only public protocol and types — no private internals, no protocol bypass.
- **Inference capability TypeScript SDK** (`sdk/typescript/inference-capability/index.ts`): Transport-neutral inference capability contract SDK. `InferenceRequest`/`InferenceResponse`/`InferenceStreamFrame`/`InferenceError` types, `InferenceOperationKind`/`TransportKind`/`ModalityKind`/`RuntimeKind` enums, `ProviderCapabilityManifest` provider declaration. `createInferenceRequest` builds requests and rejects raw secrets. `classifyInferenceError` maps cloud and local/resource errors. `InferenceStreamLifecycle` manages start→chunk→end/error/cancel/timeout lifecycle. `createProviderCapabilityManifest`/`validateProviderCapabilityManifest` build and validate provider manifests. 69 pure-TS self-tests passing. No URL/header/status-code/OpenAI-messages fields; cloud adapters are just one `transport_kind="http"` provider class.
- **Package templates**: `--template networked` generates a subprocess package with network permission declarations (`host`, `methods`, `purpose`), a `fetch` capability with `network` side effect, and an `echo` capability. Demonstrates `secretRef`, `NetworkDeclaration`, and `OutboundAuditHelper` usage. `--template streaming` generates a subprocess package with a streaming capability (`streaming: true`) and demonstrates `StreamFrameClient` faux frame lifecycle. `--template agent-runtime` generates a deterministic/no-network agent-like subprocess package with streaming run, trace summary, proposal draft, and echo capabilities, plus assistant_action and forge_panel surfaces. Uses `StreamFrameClient` (secure-execution) and `createTraceEvent`/`createProposalDraft`/`blockRawSecrets` (ygg-agent-adapter). All three templates are safe by default: no raw secrets, no implicit network access.
- **No-network readiness proof examples**: `examples/packages/faux-model-readiness/` proves the substrate shape for model-like packages (network declarations, secret_ref usage, discovery plans, faux streaming frames — no real inference). `examples/packages/faux-agent-readiness/` proves the substrate shape for agent-like packages (proposal/trace patterns, no network permissions, faux streaming trace — no real agent loop or pi runtime coupling).
- Package lifecycle events: `kernel/package.loading|starting|ready|stopping|stopped|loaded|unloaded|degraded|log`.
- Proposal lifecycle events: `kernel/proposal.created|approved|rejected|applied|failed`.

### Public protocol and transports

- Canonical request/response envelope with host-attached principal context. Callers cannot self-assert package or admin identity.
- HTTP `POST /rpc` and host JSON-RPC stdio (`ygg host-stdio`) call the same dispatcher.
- HTTP SSE event subscribe with `after_sequence` replay and live tailing for host-dev callers.
- Profile-backed `ygg host serve` autoloads packages and exposes `/rpc` plus SSE.
- WebSocket and TCP transports are reserved for future work; remote and WASM entries are reserved as first-class manifest forms with execution deferred.

### Package execution

- `rust_inproc` packages execute through a host-provided package trait and catalog. Manifests whose declared in-process provider is missing from the catalog are rejected.
- `subprocess` packages execute via JSON-RPC over stdio with handshake, invoke, invoke timeout, degraded state, restart, kill-on-unload, and stderr log capture.
- `wasm` and `remote` entries: manifest support yes, execution deferred.
- Capability routing supports explicit provider selection and simple exact / `^x.y` version constraints. Ambiguous routes are rejected unless the caller specifies `provider_package_id`.
- Hook fabric slice: deterministic ordering, package-owned handler capabilities, payload metadata mutation, veto, unload cleanup for `kernel/event.before_append|after_append` and `kernel/capability.before_invoke|after_invoke`.

### Substrate

- Asset registry: opaque `id`/`mime`/`hash`/`size`/`origin_package_id`/`metadata`, rehydratable from SQLite. Permission enforcement and content-addressed blob storage are next.
- Session fork/branch lineage records, rehydratable from the event log.
- Generic projection registry. Rebuild filters events by `kind_prefix` and `writer_package_id` and writes `kernel/projection.updated`. Package-owned projection execution is next.
- **Permission grant rehydration**: `kernel/permission.granted` and `kernel/permission.revoked` events are replayed during `hydrate_substrate_from_events` so that grants survive runtime reconstruction against the same SQLite store.
- Surface contributions: typed descriptors with version, slot, activation, required permissions, approval policy, metadata. Slots: `experience_entry`, `home_card`, `play_renderer`, `forge_panel`, `asset_editor`, `assistant_action`. Discoverable through `kernel.surface.contribution.list` and `.describe`.
- Proposal lifecycle: `kernel.proposal.create|get|list|approve|reject|apply`. Apply currently executes generic `asset.put` and `projection.rebuild` operations. Broader transactions and revert/compensation are next.

### Official packages

All ordinary packages. No kernel privilege. They live under `packages/official/` and load through normal manifests:

- `official/package-lab` — package-authoring helpers exposed as ordinary capabilities and surfaces.
- `official/schema-tools` — schema-validation helpers.
- `official/event-tools` — event filtering and inspection helpers.
- `official/composition-lab` — composition validation, launch-plan, permission-preview, surface-graph, and compat-report helpers with v2 descriptor diagnostics (capabilities, permissions, replacements, compatibility notes).
- `official/asset-lab` — generic asset preview, diff, export, and import-plan helpers.
- `official/projection-lab` — projection describe, diff, rebuild-plan, and source-event helpers.
- `official/persona-lab` — persona profile import, normalization, rendering, and compatibility diagnostics.
- `official/knowledge-lab` — structured knowledge collection normalization, matching, injection planning, and diagnostics.
- `official/context-lab` — bounded context block assembly, layer inspection, budget planning, and template rendering.
- `official/text-transform-lab` — deterministic text transform import, validation, preview, pipeline explanation, and diagnostics.
- `official/model-connector-lab` — no-network provider family metadata, profile validation, secret masking, discovery plans, and compatibility reports.
- `official/model-provider-lab` — cloud API adapter lab, not the Yggdrasil model abstraction, not an API gateway, and not privileged by the kernel. Provides no-network adapter-local request builders/profile validation across eight cloud providers (rejecting raw secrets), fake/local invoke (all eight families: OpenAI chat/responses, Anthropic messages, Gemini generateContent, OpenAI-compatible chat, OpenRouter chat/responses, DeepSeek chat, xAI chat/responses, Fireworks chat/responses; auditable outbound_request_shape), stream normalization (delta SSE, semantic SSE, typed chunk stream → StreamFrameEnvelope frames: start/chunk/progress/end/error/cancelled/timeout; all eight families; terminal_frame_consistent check; provider event input normalization), error explanation, and echo. `normalize_request` is a package-local helper, not the platform canonical inference request.
- `official/model-routing-lab` — no-inference consumer-slot binding, route planning, fallback planning, and params normalization.
- `official/assistant-lab` — assistant-action capability that returns approval-gated proposals.
- `official/pi-agent-runtime-lab` — reference agent runtime package with deterministic no-network run plans, trace summaries, proposal drafts, and echo payloads.
- `official/capability-tool-bridge-lab` — discovers capabilities, previews permissions, resolves explicit provider selection, and drafts invocation/streaming plans through kernel.capability.invoke/stream without preferring official providers.
- `official/inference-local-lab` — deterministic non-HTTP fake local inference provider proof. Proves the inference capability seam does not depend on cloud APIs, HTTP, bearer tokens, JSON provider schemas, or network access. Provides describe_capabilities (transport_kinds in_memory/local_process, network_required=false, secrets_required=false), invoke (rejects http transport, HTTP-shaped/messages-shaped fields, raw secrets; returns deterministic output, network_performed=false, transport_performed=in_memory_fake), stream (deterministic start/chunk/progress/end frames, no URL/header/status/provider_schema), explain_error (covers local/resource error classes). 5 conformance cases.
- `official/blank-experience` — minimal experience used by `ygg play-create-demo` to exercise the play-creation loop.
- `official/playable-seed` — reference playable package with entry/play/Forge/assistant surfaces.

The Forge profile (`profiles/forge-alpha.yaml`) autoloads these alongside example fixture packages.

### Web shell (`clients/web`)

- Skeletal Home/Play, Forge, and Assist surfaces over the public protocol.
- Home discovers `experience_entry` surfaces, launches sessions through the package-declared launch capability, supports session fork.
- Forge inspects packages, capabilities, assets, projections, proposals, events, and surface contributions, with package/capability inventory by provider, surface descriptor inventory by slot, composition/authoring diagnostics, manifest/template CLI guidance, and approve/apply controls for proposals.
- No official-package hardcoding. The shell is a public-protocol client like any other.
- **Text Surface Proof (Phase T1)**: A lightweight client-side text-layout adapter (`clients/web/src/text-layout`) aligned with the Pretext API shape (`prepareText`, `layoutPreparedText`, `createStreamingBuffer`). It runs without a Pretext dependency and uses a canvas-based fallback for line-breaking, line-count, and height estimates. A mock-streaming proof lives inside the Assistant Drawer: inert mock chunks display progressively with live line/height metadata and stream-lifecycle badges (`idle`/`streaming`/`ended`/`reset`). This is a UI proof only; no kernel/package/protocol changes, no real model/agent calls, no network traffic.
- **Optional Text Engine (Phase T2)**: `TextEngine` interface, engine registry, fallback engine with bounded width cache (4096 entries), and generic stream-frame-to-buffer adapter. No kernel/protocol changes.
- **Optional Pretext Engine (Phase T3)**: `PretextTextEngine` with dynamic import, runtime feature flags (`auto`/`fallback`/`pretext`), and graceful fallback. Repo builds without `@chenglou/pretext`. Assistant Drawer shows engine preference, Pretext availability, and fallback reason.
- **Forge Text Preview (Phase T4)**: Text preview helper extracting safe plain-text from event payloads, stream frames, and proposal objects. Optional `<details>` in Forge Events and Proposals with preview text, line/height estimates, and engine badge. No replacement of JSON inspectors.
- **SDK Extraction & Hardening (Phase T5)**: `sdk/typescript/text-surface` — pure TypeScript frontend SDK with `createTextSurfaceBuffer`, `applyStreamFrame`, `extractTextChunk`, `createScrollAnchor` (no `clients/web` dependency). Font-loading helper (`ensureTextSurfaceFontLoaded`, `describeFontLoadState`). Cache diagnostics (`getCacheDiagnostics` with `totalEntries`/`fontCount`/`maxEntries`/`estimatedBytes`). Self-test harness (`runTextLayoutSelfTest`) with pure TS assertions for fallback engine, registry, stream adapter, and text preview.
- **Agent Observability (Phase J5)**: `clients/web/src/agent/observability.ts` — pure UI helper that extracts agent-like observability from events, proposals, surfaces, and capabilities using generic string heuristics (no hardcoded official package; no real model or network calls). The Forge surface gains an "Agent Observability" section with cards/summary counts, a trace timeline, tool bridge diagnostics badges, and proposal explanations (reusing the T4 text preview). The Assistant Drawer gains a lightweight "Agent Readiness" panel showing discovered agent-like surfaces/capabilities counts, emphasizing no real model, no network, proposal-gated, plan-only behavior; buttons are disabled and do not launch a real agent.

### Authoring

- `ygg init-package` generates Python or TypeScript subprocess package skeletons. The TypeScript variant uses the SDK runtime under `sdk/typescript/subprocess`.
- `--template basic|experience|play-renderer|forge-panel|assistant-action|asset-editor|full-surface|networked|streaming|agent-runtime` controls generated surface descriptors. Without `--template`, `--language *-experience` auto-detects a legacy 4-surface experience mode for backward compatibility; otherwise defaults to basic. `networked` template adds network permission declarations and demonstrates `secretRef`/`NetworkDeclaration`/`OutboundAuditHelper` usage. `streaming` template adds a streaming capability and demonstrates `StreamFrameClient` faux frame lifecycle. `agent-runtime` template generates an agent-like package with streaming run/trace/proposal/echo capabilities and assistant_action/forge_panel surfaces, using the `ygg-agent-adapter` SDK.
- `--language typescript-experience` (without `--template`) still generates the original 4-surface experience descriptors for backward compatibility.
- `ygg init-composition` and `ygg composition check` provide a local composition descriptor flow with v2 fields (title, description, optional packages, required capabilities, default activation, permission expectations, replacement candidates, compatibility notes). `composition check` prints structured diagnostics: loaded required/optional packages, surfaces by slot, capabilities, entry activation, missing required surfaces/capabilities (fail), and warnings for missing optional packages.
- `ygg package check` and `ygg package conformance` validate generated packages locally. `ygg package check` prints structured diagnostics: entry kind, trust level, capability count, surfaces by slot, permissions summary, sandbox policy summary, and warnings for packages with no capabilities or no surfaces.
- `ygg package reload <manifest>` loads a package into an in-memory runtime, restarts it (subprocess only), prints before/after status and logs count, then unloads. Uses existing Runtime::restart_package path; no new protocol methods.
- `ygg package run-fixture` invokes all declared non-streaming capabilities with deterministic canned inputs and prints a structured JSON summary.
- `ygg play-create-demo` orchestrates the blank play-creation loop end-to-end through ordinary public-protocol calls.

### Code organization

- `crates/ygg-cli/src/main.rs` is a thin entry point. CLI types live in `cli.rs`; commands live under `commands/`; package generation templates live under `templates/`; conformance cases live under `conformance/` domain modules.
- `crates/ygg-runtime/src/runtime/` owns runtime domain behavior across session, events, packages, capabilities, hooks, permissions, assets, branches, projections, proposals, and protocol dispatch modules; `runtime/mod.rs` preserves the public `Runtime<S>` API and re-exports moved public request/record types.
- Protocol method metadata and dispatch share the `KernelMethod` source of truth, with unit coverage for registry/dispatch consistency.
- `crates/ygg-runtime/src/inproc.rs` retains the in-process package API and delegates official lab behavior to focused modules under `crates/ygg-runtime/src/inproc/`.
- `crates/ygg-runtime/src/inproc/common.rs` routes shared official in-process handlers by provider package and local capability name rather than suffix-only fallback.
- This split is behavior-preserving and exists to keep future package, conformance, and handler growth reviewable.

### Conformance

- `cargo run -p ygg-cli -- conformance` runs 150 named CLI cases covering: sessions, events, packages, capabilities, hooks, schemas, principals, permissions, subprocess execution, host transports, surfaces, proposals, official packages, composition-lab, asset-lab, projection-lab, persona-lab, knowledge-lab, context-lab, text-transform-lab, model-connector-lab, model-provider-lab, model-routing-lab, pi-agent-runtime-lab, capability-tool-bridge-lab, inference-local-lab, in-process package fallback hardening, playable-seed, blank play-creation loop, generated package authoring, composition descriptors, package check diagnostics, package reload smoke, third-party replacement proof, permission grant rehydrate, secret_ref validation, raw-secret blocking, official-package no-secret-bypass, **env secret resolver (allowed/denied/missing-no-leak; deny-all default; allowlist-only; no raw value leak)**, network permission enforcement with audit, network policy checker pure function, **outbound executor boundary (denied request never reaches executor, policy/executor mismatch fails closed, allowlisted fake executor with network_performed:false, raw body not in audit, secret_refs as refs only, host mismatch redirect denied)**, **model provider invoke adapters (OpenAI chat/responses, Anthropic messages, Gemini generateContent fake/local invoke, raw credential rejected, unsupported family diagnostic, auditable outbound_request_shape)**, **model provider outbound shape fake executor (three-provider host/method/path/secret_ref shapes pass outbound boundary, call_count=3, executor_kind Fake)**, **model provider stream normalization (eight families delta SSE/semantic SSE/typed chunk stream normalized to start/chunk/progress/end frames, terminal_frame_consistent, provider event input normalization, no raw secret echo)**, streaming/cancellation lifecycle, generated template conformance, no-network readiness proof, **live HTTP outbound executor (default DenyAll still effective, non-HTTPS URLs fail closed with no network, response shape contains no raw body/header/secret)**, **kernel.outbound.execute public protocol (package principal determined from context (no spoofing), FakeOutboundExecutor + allowed network declaration succeeds with audit, spoofed package_id rejected, no network permission denied with executor not called, secret_refs as references only with no raw secret in response)**, **L4: outbound secret_headers parsing verified (secret_headers params format parsed correctly, raw secret never in response), local loopback HTTP server secret injection conformance (Authorization header actually arrives at server, raw secret not in protocol response/audit/log), DeepSeek SSE stream normalize canary (delta_sse start→chunk→end lifecycle, terminal_frame_consistent, no raw secrets), opt-in live DeepSeek conformance (default skip, only when YGG_LIVE_MODEL_TESTS=1 + DEEPSEEK_API_KEY), canary DeepSeek profile shape (normalize_request endpoint/dialect/stream_family correct, secret_ref placeholder no raw key)**, **L5 OpenAI/Anthropic/Gemini live adapter conformance (OpenAI chat loopback with Authorization bearer, OpenAI responses loopback, Anthropic messages loopback with x-api-key secret + anthropic-version static header, Gemini generateContent loopback with x-goog-api-key, missing secret fails closed, provider normalize_request alignment, no raw secret leak across all providers, static_headers safe allowlist, static_headers block secret-bearing names)**.
- Plus crate and service unit tests under `cargo test --workspace`.
- `tsc -p clients/web/tsconfig.json --noEmit` checks the web shell.

## What is partial

- Capability invocation lifecycle events (`kernel/capability.invoked|completed|failed`) reserved in contract; not emitted yet.
- Streaming protocol dispatch is partial (stream start/cancel lifecycle works; real network streaming deferred).
- Package-principal `event.subscribe` permissions.
- Hook handler timeout/error audit for package-owned handlers.
- Persisted capability provider selection policy beyond per-invocation explicit selection.
- Richer resource policy coverage (filesystem enforcement matrices) — Phase S4+ target.
- Content-addressed asset blob storage and package-principal asset permission checks.
- Package-owned projection execution.
- Richer crash monitoring and health-check beyond lifecycle events.
- Broader transport parity coverage in conformance beyond the current core protocol dispatcher and service tests.
- Richer TypeScript SDK packaging beyond the current thin subprocess helper and secure-execution helpers.
- Full `kernel.session.get|list`, `kernel.package.describe`, `kernel.capability.describe`, `kernel.extension_point.describe`, `kernel.host.principal`, `kernel.host.ping` route exposure.
- Production secret vault integration (Phase S1 provides the contract, `DenyAllSecretResolver`, and `EnvSecretResolver`; full vault integration deferred).
- Network permission declarations and outbound audit/redaction records (Phase S2 — implemented).

## What is deferred

These are non-goals for the kernel and are expected to ship as ordinary packages or future work:

- Conversational runtime, prompts, models, sampling, message/turn semantics.
- Memory model, retrieval, summarization, agent loop, director.
- World, scene, actor, rule, dice, inventory semantics.
- SillyTavern resource and behavior compatibility (see `docs/tavern/TAVERN_COMPAT.md`).
- Real agent loops, production-grade live model calls, and memory systems (agent-like infrastructure plus provider adapter/fake-local invoke are complete; see `docs/architecture/PI_INTEGRATION.md`, `docs/guides/AGENT_PACKAGE_AUTHORING.md`, and `docs/guides/MODEL_PROVIDER_INTEGRATION.md`).
- External game engine bridges (UE5, Godot, Unity, web clients).
- Marketplace, package signing, dependency resolver.
- Final UI visual design, full Studio, ComfyUI-like node editors.
- WASM and remote package execution.

## How to verify this snapshot

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

If any of the above fails, this document is wrong; the code is right. Update this document.

## Where to read next

- `docs/CHARTER.md` — what does not change.
- `docs/architecture/VISION.md` — what the platform is for.
- `docs/architecture/ARCHITECTURE.md` — kernel-and-packages layering.
- `docs/architecture/PLATFORM_KERNEL.md` — what the kernel does and does not do.
- `docs/architecture/CAPABILITY_PACKAGE.md` — package contract.
- `docs/architecture/EVENT_MODEL.md` — opaque event log.
- `docs/architecture/EXTENSION_POINTS.md` — hook contract.
- `docs/architecture/RUNTIME_LIFECYCLE.md` — kernel-side lifecycles.
- `docs/protocol/PROTOCOL_V0.md` — public protocol.
- `docs/spec/KERNEL_V0_ALPHA_CONTRACT.md` — executable alpha contract matrix.
- `docs/spec/CONFORMANCE_MATRIX.md` — hostile conformance roadmap.
- `docs/product/PLAY_CREATION_MODEL.md` — play-creation product stance.
- `docs/guides/AGENT_PACKAGE_AUTHORING.md` — agent-like capability package authoring guide.
- `docs/guides/MODEL_PROVIDER_INTEGRATION.md` — multi-provider cloud API integration guide.
- `docs/guides/INFERENCE_CAPABILITY_AUTHORING.md` — transport-neutral inference capability package authoring guide.
- `docs/roadmap/CREATIVE_INFERENCE_CAPABILITY_ALPHA.md` — current Creative Inference Capability Alpha temporary plan.
- `docs/roadmap/NEXT_STEPS.md` — current and upcoming phases.
