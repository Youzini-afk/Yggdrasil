# Platform status

> [English](./ALPHA_STATUS.en.md) · [中文](./ALPHA_STATUS.md)

A snapshot of where Yggdrasil is right now, refreshed whenever a milestone closes. Every line below has code and a conformance case behind it, unless explicitly marked partial or deferred.

For vision and principles, see [`CHARTER.md`](CHARTER.en.md), [`architecture/VISION.md`](architecture/VISION.en.md), and [`product/PLAY_CREATION_MODEL.md`](product/PLAY_CREATION_MODEL.en.md). For what's next, see [`roadmap/NEXT_STEPS.md`](roadmap/NEXT_STEPS.en.md).

## Summary

- **Conformance:** 442 named CLI cases pass, plus crate and service unit tests; 144 v1 schemas validate (80 methods + 57 events + 7 top-level).
- **Charter discipline:** content-free kernel; no privilege for official packages; public protocol only; equal entry forms; capability handles, binding injection, Path A / Path B, the conformance kit, and generated SDKs are implemented; trusted paths block raw secrets and use manifest-declared `secret_ref` everywhere; permission grants rehydrate; network permissions are audited and redacted; generic streaming and cancel lifecycle; outbound execution has a boundary, deny-all by default; public HTTPS outbound uses the same host-policy / audit / redaction boundary; unary outbound, SSE/NDJSON/raw streams, and WebSocket all emit completion audit events.
- **Code health:** the CLI, runtime domain behavior, protocol dispatch, in-process handlers, and the event store are all split by domain. We're not stacking more onto single files.
- **Human-testing substrate:** install warnings and schema shapes are stable; native project install now flows source → store → nested manifests/profile autoload → project registry → project dist → `/surface-bundles/projects/<project_id>/...`; `surface_bundle` is a static, non-executing entry; `dist/` is included in `tree_hash`, store schema migration clears old stores, and install/update/uninstall garbage-collect orphan stores; `official/install-lab` provides `check_for_updates` / `update_project`, and both CLI `yg update` and the web project console route through it; the Surface bridge has converged on allowlists, stream ownership, redacted diagnostics, secret-input cleanup, CSP/CORS hardening, and typed `allowed_capability_ids`; the self-hosted deployment substrate is implemented: target / exec / port / proxy primitives, ygg-service HTTP/WebSocket reverse proxy, LiveLocalExecExecutor, `official/docker-runtime-lab`, and explicit web Deploy broker.

The platform substrate is in place. From here, real project deployment, human testing, and AI-native experiences pull the remaining substrate work.

## Kernel

- Content-free sessions, append-only opaque events, manifest-driven packages, capability fabric, hook fabric, surface contributions, the proposal lifecycle, and the asset / branch / projection substrate.
- A SQLite event log with monotonic per-session sequence numbers and a rehydratable substrate.
- A JSON Schema subset validates capability I/O and package-declared event payloads.
- Principals: `host_admin`, `host_dev`, `package`, `human`, `assistant`, `anonymous`. Human and assistant principals get scoped grants.
- Audit events: `kernel/v1/permission.granted|revoked|denied`, `kernel/v1/package.*` lifecycle, `kernel/v1/proposal.*` lifecycle.
- Persistent grants: grant / revoke events rehydrate inside a SQLite-backed runtime.
- Contract V1 is the public platform spec: 80 protocol methods, 57 event kinds, and 144 JSON Schemas. `kernel.v1.cap.*`, `kernel.v1.audit.package`, capability handles, binding injection, Path B, the conformance kit, and SDK generation are implemented.

## Secure execution

- **`secret_ref` references:** `secret_ref:<vault>:<key>`, `secretRef:`, `secret-ref:`, and `host:` prefixes are all supported. Packages refer to secrets through these references; raw values never appear in events, proposals, logs, or audit records.
- **Environment-variable resolver:** a host-owned resolver with an explicit allowlist. Deny-all by default; an env name has to be allowed before it can be resolved. Errors carry only the env name, never the raw value.
- **Local encrypted secret store:** `secret_ref:store:NAME` resolves through `StoreSecretResolver` from `~/.yggdrasil/secrets.dat`; `secret_ref:project:NAME` reads the project-level store first and then falls back to the platform store according to `secret_policy`; stores use age (X25519), with a master key from OS keyring (deferred) or a 0600 local key file.
- **Raw secret blocking:** proposal operations and expected effects, plus asset metadata, are scanned conservatively. Obvious API keys, tokens, and password fields are rejected. Asset content and ordinary prose aren't scanned, to avoid false positives on user content.
- **Network permission declarations:** `permissions.network` in a manifest supports both flat `hosts` (backward compatible) and structured `declarations` with `host`, `methods`, and `purpose`. A package without a declaration can't reach the network. Official packages don't bypass.
- **Outbound audit and redaction:** every outbound request produces an audit record holding only the principal, the package id, the capability id, the destination host, the method, the purpose, the redaction state, and the `secret_ref`s used. Raw bodies, headers, prompts, and responses are never recorded.
- **Outbound executor boundary:** content-free HTTP and WebSocket executor traits. Default is deny-all (fail closed). They can switch to fake executors (with fixtures, used by conformance) or live executors (HTTP uses reqwest + rustls; WebSocket uses tokio-tungstenite + rustls; both off by default; HTTP is HTTPS-only; WebSocket is WSS-only; redirect fail closed). Secret headers are injected at execution time only — never into audit, response, or `Debug`. Real live model / WebSocket outbound requires explicit opt-in through profile and environment variables; default conformance does not use the network, and real WebSocket smoke also requires `YGG_LIVE_WEBSOCKET_TESTS=1`.
- **Protocol methods:** `kernel.v1.outbound.audit` lists outbound audit events for a package; `kernel.v1.outbound.execute` lets ordinary packages issue unary outbound requests through the host executor; `kernel.v1.outbound.stream` provides SSE/NDJSON/raw streaming outbound; `kernel.v1.outbound.websocket.open|send|close` provides bidirectional WebSocket outbound.
- **Completion audit events:** `kernel/v1/outbound.execute.completed`, `kernel/v1/outbound.stream.completed`, and `kernel/v1/outbound.websocket.completed` cover all three outbound primitives; events record only status, counts, duration, executor kind, network_performed, redaction state, and `secret_ref` references.
- **Streaming lifecycle:** the stream registry tracks in-flight streaming invocations and emits `kernel/v1/stream.started|chunk|progress|ended|error|cancelled|timeout` in order. Cancel and timeout block further chunks. Non-streaming capabilities are rejected.

## Public protocol and transport

- A canonical request / response envelope carrying a host-bound principal context. Callers can't claim to be a package or admin.
- The same dispatcher handles HTTP `POST /rpc` and host JSON-RPC stdio (`ygg host-stdio`).
- Event subscription via SSE, with `after_sequence` replay and live tailing.
- Profile-driven `ygg host serve` autoloads packages and exposes both `/rpc` and SSE.
- TCP transport is reserved for later. WASM and remote entries are first-class manifest forms; execution is deferred.

## Package execution

- `rust_inproc` packages run through host-provided traits and a catalog. Manifests that declare an in-process provider but aren't in the catalog are rejected.
- `subprocess` packages run over JSON-RPC on stdio: handshake, invoke, timeouts, degraded state, restart, kill-on-unload, stderr capture.
- `wasm` and `remote` entries: manifests support them; execution is deferred.
- Path A (`entry.contract: "v1"`) receives capability-handle bindings and permission enforcement; Path B (`entry.contract: "none"`) runs self-contained with no v1 authority, while lifecycle remains observable.
- Capability routing supports explicit provider selection and simple exact / `^x.y` version constraints. Ambiguous routes are rejected unless the caller supplies `provider_package_id`.
- Hook fabric: deterministic ordering, package-owned handler capabilities, payload metadata mutation, veto, unload cleanup. Covers `kernel/v1/event.before_append|after_append` and `kernel/v1/capability.before_invoke|after_invoke`.

## Substrate

- Asset registry: opaque `id`, `mime`, `hash`, `size`, `origin_package_id`, `metadata`. Rehydrates from SQLite. Permission enforcement and content-addressed blob storage are next.
- Session fork / branch lineage rehydrates from the event log.
- Generic projection registry. Rebuilds filter the event log by `kind_prefix` and `writer_package_id` and write `kernel/v1/projection.updated`. Package-owned projection execution is next.
- Project runtime: `ProjectDescriptor`, `ProjectRegistry`, `~/.yggdrasil/projects/<id>/` layout, project-level secret policy, Home project cards, per-project storage summaries, redacted package-failure summaries, and `yg project list/info/status/start/stop` are implemented.
- Deployment runtime: `kernel.v1.target.*`, `kernel.v1.exec.*`, `kernel.v1.port.*`, and `kernel.v1.proxy.*` are implemented; default is deny-all; profiles can opt into `LiveLocalExecExecutor`; ports are loopback-only; proxy upstreams must reference active port leases; ygg-service provides HTTP/WebSocket reverse proxy. The web project console can explicitly Deploy / Stop Docker based on `project.metadata.deployment.docker`.
- Surface contributions: descriptors with version, slot, activation, required permissions, approval policy, and metadata. Slots are `experience_entry`, `home_card`, `quick_action`, `workshop_card`, `play_renderer`, `forge_panel`, `asset_editor`, and `assistant_action`. `quick_action`, `workshop_card`, and `home_card` entries with `metadata.shell_schema_version: 1` are structured shell descriptors: the web shell reads only bounded text, icon hints, order, and same-package targets, then renders them itself. It does not load package JavaScript, parse HTML, or mount iframes for those entries. Complex project surfaces still use `surface_bundle` plus sandboxed iframe hosting. Discoverable via `kernel.v1.surface.contribution.list` and `.describe`.
- Surface bundles: `surface_bundle` is a static browser-bundle entry in the manifest, not an executable package entry; installed project bundles are served by the host as same-origin static files under `/surface-bundles/projects/<project_id>/...`. `dist/` participates in `tree_hash`, so browser-bundle-only changes trigger updates; project dist refreshes through a temporary directory plus atomic replacement.
- Proposal lifecycle: `kernel.v1.proposal.create|get|list|approve|reject|apply`. `apply` currently runs the generic operations `asset.put` and `projection.rebuild`. Broader transactions and revert / compensation are next.

## Package installation and project model

| Capability | Status |
|---|---|
| `manifest.requires` field | implemented |
| Lockfile schema (`yggdrasil.lock.v1`) | implemented |
| `official/git-tools-lab` (gix-based) | implemented |
| `official/integrity-lab` (sequoia GPG + sha256) | implemented |
| `official/install-lab` orchestrator | implemented |
| `yg install` / `uninstall` / `list-installed` / `update` / `lockfile` CLI | implemented |
| `~/.yggdrasil` filesystem convention | implemented |
| Interactive consent prompt | implemented |
| Static conformance integration (warning by default; `--strict` blocks) | implemented |
| GPG signature verification (off by default; `--require-signed` enables) | implemented |
| Cycle detection | implemented |
| Real GitHub smoke (opt-in) | implemented |
| `dist/` included in `tree_hash` | implemented |
| Store schema migration clears old store | implemented |
| Orphaned store GC (after install / update / uninstall) | implemented |
| `official/install-lab/check_for_updates` | implemented |
| `official/install-lab/update_project` | implemented |
| `official/secret-store-lab` encrypted storage | implemented |
| `official/docker-runtime-lab` (Docker container lifecycle, bollard) | implemented |
| `StoreSecretResolver` + `CompositeSecretResolver` | implemented |
| age (X25519) encryption + 0600 file permissions | implemented |
| OS keyring integration | deferred (libdbus-sys system dep) |
| `yg secret put / list / delete` CLI | deferred |
| Sigstore verification | deferred |
| Tauri UI install path | deferred |
| Auto-update daemon | deferred |
| Binary package distribution | deferred |
| Project as first-class runtime concept | implemented |
| `ProjectDescriptor` + `ProjectId` + `ProjectType` + `SecretPolicy` | implemented |
| `~/.yggdrasil/projects/<id>/` filesystem layout | implemented |
| `secret_ref:project:NAME` with platform fallback | implemented |
| `ProjectRegistry` (in-memory + disk scan) | implemented |
| `ProtocolContext.session_id` propagation | implemented |
| Install detection (native vs external) | implemented |
| External project wizard (wrap / workspace) | implemented |
| `yg project list/info/status/start/stop` | implemented |
| `yg uninstall` archival prompt | implemented |
| `kernel.v1.project.list/get/start/stop/status` | implemented |
| `kernel/v1/project.installed/started/stopped/uninstalled` | implemented |
| Home surface project cards | implemented |
| YdlTavern `project.yaml` | implemented |
| Native project install into profile, project registry, and project dist | implemented |
| `surface_bundle` static entry and installed project bundle route | implemented |
| typed `allowed_capability_ids` bridge declaration | implemented |
| CLI `yg update` routes through install-lab project update | implemented |
| Multi-tenant `project_id` in `ProtocolContext` | deferred |
| Project archive auto-cleanup beyond 30 days | deferred |

Install defaults are relaxed to the cargo / npm / pip technical baseline: HTTPS-only, content hashing, and atomic writes are always on; signature verification and conformance blocking are opt-in through `--require-signed` / `--strict`.

## Real model end-to-end path

| Capability | Status |
|---|---|
| `huggingface-fetcher` tests passing | implemented |
| Surface bundle resolution metadata-driven | implemented |
| `kernel.v1.surface.resolve_bundle` | implemented |
| host `/surface-bundles/<prefix>/<file>` route | implemented |
| `/surface-bundles/projects/<id>/<file>` route | implemented |
| `project.start` opens project session + sets `metadata.project_id` | implemented |
| `project.start` returns `session_id` + `already_running` | implemented |
| `project.get` / `status` return `running_session_id` | implemented |
| `project.stop` emits + closes project session | implemented |
| Surface receives `session_id` via `initialProps` | implemented |
| `TavernProvider.sendMessage` invokes engine `model.live_call` | implemented |
| API Connections drawer scope toggle (platform / project) | implemented |
| Engine manifest declares `secret_ref:project:*` | implemented |
| Surface streaming response UX | implemented |
| Surface-host stream postMessage protocol | implemented |
| Surface bridge allowlist / stream ownership / diagnostics redaction / secret input cleanup / CSP/CORS hardening | implemented |
| `streamCapability` helper (YdlTavern host-rpc) | implemented |
| `AsyncIterable<StreamFrame>` consumption + iterator early-return cleanup | implemented |
| `cancelGeneration` action + Stop button | implemented |
| Multi-concurrent generation in single chat | deferred |
| Token-rate UI | deferred |
| Realtime / WebSocket streaming UX | deferred |

## Official capability packages

All ordinary packages, no kernel privilege. They live in `packages/official/` and load through ordinary manifests.

**Platform foundation**

- `package-lab`, `schema-tools`, `event-tools`, `composition-lab`, `asset-lab`, `projection-lab`, `assistant-lab`.
- Package installation foundation: `official/git-tools-lab`, `official/integrity-lab`, and `official/install-lab`.

**Creative capability families**

- `persona-lab`, `knowledge-lab`, `context-lab`, `text-transform-lab`.

**Model integration**

- `model-connector-lab` — offline provider metadata, profile validation, secret masking, discovery plans, compatibility reports.
- `model-provider-lab` — a cloud-API adapter lab covering OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, and Fireworks. Provides request builders, fake invocations, stream normalization, live loopback shapes, and per-provider quirks. It's not a platform model abstraction and not an API gateway.
- `model-routing-lab` — consumer-slot binding, route planning, fallback planning, and parameter normalization, with no inference of its own.

**Agents and inference**

- `pi-agent-runtime-lab` — a reference agent package: deterministic, no-network run plan, trace summary, proposal draft, and echo.
- `capability-tool-bridge-lab` — capability discovery, permission preview, explicit provider selection, invocation / streaming plans, covering nested delegation, target-branch writes, prompt injection, secret exfiltration, outbound expansion, and large-output redaction.
- `agentic-forge-lab` — the core of Agentic Forge: package-owned run lifecycle, working state, plan graph, scratch branch / candidate / compare / promote, inference nodes (deterministic / recorded / cloud-adapter plan / local fake), replay, output validation, and a 9-class failure taxonomy.
- `inference-local-lab` — a local fake inference provider that proves the inference seam doesn't depend on cloud APIs, HTTP, or bearer tokens.
- `inference-playtest-lab` — the Ygg-native "inference → proposal → inspect → approve / reject → apply → fork" vertical slice.

**Experience**

- `experience-runtime-lab` — the experience runtime contract: experience descriptors, state projection, checkpoint, recovery, and Play / Forge / Assist surface bindings.
- `playable-creation-board` — the first real playable vertical slice. Package-owned board / module / constraint / marker state, 14 capabilities, 4 surfaces.
- `experience-observability-lab` — package-owned observability: session health, package health, agent run health, proposal causal chain, cost / latency summaries, failure breadcrumbs, guardrail summaries.
- `memory-lab` — long-term memory and knowledge: record, retrieve, retrieval trace, approval-gated update, correction, forget / redaction, branch view, provenance.
- `sharing-lab` — sharing and distribution: composition bundle import / export, branch / session bundle manifests, package-set lockfiles, compatibility reports, AI disclosure metadata, read-only share manifests, async fork plans. No marketplace, no billing, no signing network.
- `playable-seed`, `blank-experience` — reference and minimum experiences.

**Storage and external projects**

- `storage-lab` — a preview of storage / data contracts: layered model, backend-class candidates, package-level state stores, document CRUD previews, content-addressed blob contract proofs, projection materialization, retrieval / vector / multimodal provider contracts.
- `tdb-retrieval-lab` — TDB as a retrieval / multimodal provider contract; not the event-log authority.
- `project-intake-lab` — external-project classification, stack detection, npm lifecycle risk, workspace plans, adapter plans, wrapper / fixture / readiness previews. No network, no filesystem.
- `workspace-lab` — workspace action policy boundary, a 10-action taxonomy, deny-by-default fake executor, deterministic fixture workspace.

**Third-party replacement proofs**

- `thirdparty/playable-seed`, `thirdparty/agent-runtime`, `thirdparty/agentic-forge`, `thirdparty/memory-lab` — show that each official package can be replaced by a third party with no priority for the official version.

The Forge profile (`profiles/forge-alpha.yaml`) autoloads these and the example fixture packages.

## TypeScript SDKs

Under `sdk/typescript/`:

- `subprocess` — subprocess-package scaffolding and template runtime.
- `secure-execution` — `secret_ref` construction and validation, network declarations, outbound audit, faux stream-frame client.
- `inference-capability` — transport-neutral inference contract.
- `model-provider-adapter` — cloud-provider adapter helpers.
- `ygg-agent-adapter` — maps Ygg capabilities into pi-style tools.
- `agentic-forge` — run lifecycle, plan graph, working state, candidate / compare / promote, inference nodes, tool bridge v2 helpers.
- `experience-runtime` — experience runtime types and constructors.
- `text-surface` — frontend text-surface helpers (streaming buffer, frame adapter, scroll anchor, font loading).

`text-surface`, `agentic-forge`, `inference-capability`, and several others ship pure-TS self-tests.

## Contract v1 and SDK generation

- `docs/spec/KERNEL_V1_CONTRACT.md` is the public platform spec.
- `docs/spec/v1/schemas/` is the single source of truth for SDKs and conformance: 80 methods, 57 events, 7 top-level schemas, 144 total.
- `sdk/typescript/kernel-sdk/` and `sdk/rust/yg-kernel-sdk/` are generated from schemas; the TypeScript package can be consumed through npm, workspace path, or independent codegen.
- `yg conformance package --contract v1 --path <package>` provides 8 third-party package acceptance checks.

## Package templates

`ygg init-package --template <name>`: `basic`, `experience`, `play-renderer`, `forge-panel`, `assistant-action`, `asset-editor`, `full-surface`, `networked`, `streaming`, `agent-runtime`, `experience-runtime`, `playable-board`, `playable-experience`. Generated packages are safe by default — no raw secrets, no implicit network.

## Web shell (`clients/web`)

The platform user-facing chrome — Home, Settings, Install flow, Project frame, and the toast system. Built as a React 19 + Tailwind v4 + Motion + Radix + Phosphor SPA, bundled by Vite with route- and modal-level lazy splitting. Visual rules and the design system live in [`design/PLATFORM_UI_DESIGN.md`](design/PLATFORM_UI_DESIGN.en.md); detailed shell documentation is in [`../clients/web/README.md`](../clients/web/README.md).

- **Home:** project shelf (card grid + status pills + Hero + utility strip + activity timeline + workshop utilities bento), backed by `kernel.v1.project.list`; disk usage comes from project `storage_summary`. Home also consumes structured shell descriptors: built-in quick actions remain, and package-contributed `quick_action`, `workshop_card`, and schema-versioned `home_card` entries are discovery affordances rendered by the platform. Package actions are discovery-only in this slice and do not bypass proposal / permission / audit. `⌘N` opens the Install modal.
- **Settings:** five panels, all wired to real data.
  - API Connections — `official/secret-store-lab/{list,put,delete}_secret` plus health. The UI never reads raw secret values; secret-edit modals wipe their input state on close.
  - Installed Packages — `kernel.v1.package.list` plus the project flag, with Cmd/Ctrl+F focus.
  - Profiles — `kernel.v1.host.diagnostics` (active profile, packages_loaded, network allowlist).
  - Storage — storage-area summary plus the live event-store kind (sqlite / postgres / memory), without exposing host absolute paths in the Web UI.
  - About — platform identity, license, links, gratitude.
- **Install / Update flow:** the Install modal calls `official/install-lab` (`resolve_plan` / `detect_kind` / `execute_plan`) through `kernel.v1.capability.invoke`. Native projects take the fast path; external projects branch into a wrap-vs-workspace wizard. The project console shows bundle / package / event diagnostics and exposes updates through `check_for_updates` / `update_project`. There is no `kernel.v1.install.*`.
- **Project Frame:** Home opens projects in standalone `/project/<id>` tabs. The project page has no platform topbar or back button; it fills the viewport with the sandboxed iframe that mounts the project's own UI. Closing the tab does not stop the project. `⌘ .` / `Ctrl .` stops the current project from the project tab.
- **Failure Modal:** Deep Rust accent stripe, two-column diagnosis / impact, redacted stderr panel (with Copy log), and Restart / Stop-and-uninstall / Close actions. Data comes from `kernel.v1.package.list/status/logs`; raw logs are not copied into the UI.
- **Toast system:** five variants (info/success/warning/error/progress), bottom-right queue; honors `prefers-reduced-motion`.
- **Responsive and dark mode:** explicit `data-theme` switch (system/light/dark); `@custom-variant dark` binds Tailwind's `dark:` to that attribute; the modal overlay uses a dedicated `--color-overlay` token that does not flip with theme; `prefers-reduced-motion` collapses motion; `:focus-visible` paints a keyboard navigation ring.
- **SurfaceHost:** mounts third-party web surface bundles through sandboxed iframes; no kernel access by default; only host-configured bridge methods reach the public protocol. The bridge limits callable authority with typed `allowed_capability_ids` and method allowlists, binds stream subscriptions to the owning surface, redacts diagnostics/logs, clears secret input state on close, and keeps same-origin static bundle boundaries through CSP/CORS. Streaming subscription bridges `kernel/v1/stream.*` through postMessage.
- **No hardcoded official packages — the shell is a public-protocol client like any other.**

## Desktop and releases

- `clients/desktop` provides a Tauri 2.x wrapper. Production embeds `clients/web/dist`; development points at the Vite dev server. v0 does not start `ygg-cli host serve`; users still run the host separately.
- GitHub Actions CI and the `v*` tag release workflow are in place, building cross-platform Tauri installers and creating a draft release. `scripts/release-version.sh` synchronizes Cargo, the web package, the desktop package, and Tauri config.
- Build notes are in [`../BUILDING.md`](../BUILDING.md); changes are in [`../CHANGELOG.md`](../CHANGELOG.md). Signing, notarization, and auto-update are not enabled.

## Authoring flow

- `ygg init-package` generates Python or TypeScript subprocess scaffolding. `--template` chooses the surface descriptors. `--language *-experience` without `--template` still generates the legacy 4-surface experience for back-compat.
- `ygg init-composition` plus `ygg composition check` covers the local composition flow with v2 fields (title, description, optional packages, required capabilities, default activation, permission expectations, replacement candidates, compatibility notes).
- `ygg package check` prints structured diagnostics: entry kind, trust level, capability count, surfaces by slot, permission summary, sandbox policy. Warns on packages with no capabilities or no surfaces.
- `ygg package conformance` validates a generated package locally.
- `ygg package reload <manifest>` loads the package into an in-memory runtime, restarts (subprocess only), shows before / after status and log counts, and unloads.
- `ygg package run-fixture` invokes every non-streaming capability with deterministic fixture input and prints a JSON summary.
- `ygg play-create-demo` runs the blank play-creation loop end to end.
- `ygg perf baseline` runs deterministic baseline measurements (in-process invoke, official capability invoke, event store append / list / range, composition check, profile load, subprocess echo) in text or JSON. See [`performance/BASELINE.md`](performance/BASELINE.en.md).

## Code organization

- `crates/ygg-cli/src/main.rs` is a thin entry. CLI types live in `cli.rs`, commands under `commands/`, and package templates under `templates/`. The conformance runner and case registry are split: `conformance/runner.rs` owns `--list`, `--case`, `--tag`, `--fail-fast`, and `--slowest`; `conformance/registry/` registers the 442 `ConformanceCase { id, tags, run }` entries by domain.
- `crates/ygg-cli/src/schema_export/` owns v1 schema export; `src/bin/export-schemas.rs` is a thin entry. Generated files still come from the exporter only — SDKs and schemas are not hand-edited.
- `crates/ygg-runtime/src/runtime/` splits runtime behavior into session, events, packages, capabilities, hooks, permissions, assets, branches, projections, and proposals. `runtime/protocol_dispatch.rs` is now the public router facade; concrete public-protocol handlers live under `runtime/protocol/` by domain. `runtime/mod.rs` keeps the public `Runtime<S>` API.
- Protocol metadata and dispatch share a single source of truth (`KernelMethod`), with a registry / dispatch consistency unit test.
- `crates/ygg-runtime/src/inproc/` splits official-package behavior by domain; `official/install-lab` is split into `install_lab/` modules (types/source/planner/executor/layout/project_kind/fs_copy). The shared helper routes by provider package plus local capability name, not suffix-only fallback.
- `clients/web` Home and Install flow are split into page shells plus hooks/helpers/step components. The UI still uses public protocol only and does not read the local filesystem or private runtime state.

These splits don't change behavior — they keep the codebase reviewable as more packages, conformance cases, handlers, and UI flows land.

## Conformance

`cargo run -p ygg-cli -- conformance` runs 442 named CLI cases. Flags:

- `--list` — list ids and tags.
- `--case <pattern>` — substring filter.
- `--tag <tag>` — filter by tag.
- `--fail-fast` — stop at the first failure.
- `--slowest <N>` — report the slowest N.

Every case has tags (runtime / event / capability / package / subprocess / official / network / outbound / stream / agentic / experience / memory / sharing / secret / composition / replacement / surface / protocol / permission / hook / host / asset / projection / substrate / storage / live / external_project / project_intake / workspace_lab / retrieval, and so on). See [`performance/CONFORMANCE_FEEDBACK.md`](performance/CONFORMANCE_FEEDBACK.en.md).

Plus crate and service unit tests via `cargo test --workspace`, and `npm run check --prefix clients/web` / `npm run build --prefix clients/web` for the web shell.

## Partial (started, not finished)

- `event.subscribe` permission for package principals.
- Timeout / error audit for package-owned hook handlers.
- Persistent capability-provider selection policy beyond explicit per-call selection.
- Richer resource policy (filesystem enforcement matrix).
- Content-addressed asset blob storage and package-principal asset permissions: stable content-address helpers and metadata conventions are done; full blob storage and runtime enforcement aren't.
- Package-owned projection execution.
- Richer failure monitoring and health checks.
- Broader transport consistency coverage.
- Desktop release code signing / notarization, auto-updater, real app icons, and desktop-wrapper management of the host subprocess.
- Surface lifecycle callbacks such as `onClose` and `onProposalDraft`, plus a cross-origin surface-bundle allowlist.
- Full surfacing of `kernel.v1.session.get|list`, `kernel.v1.package.describe`, `kernel.v1.capability.describe`, `kernel.v1.extension_point.describe`, `kernel.v1.host.principal`, `kernel.v1.host.ping`.

## Deferred (explicitly out of kernel scope)

These will arrive as ordinary packages or future work — not as kernel features:

- Conversation runtime, prompts, models, sampling, message / turn semantics.
- Memory models, retrieval, summarization, agent loops, directors.
- World, scene, character, rule, dice, inventory semantics.
- SillyTavern compatibility lives in the YdlTavern integration project on top of Yggdrasil (see [`tavern/TAVERN_COMPAT.md`](tavern/TAVERN_COMPAT.en.md)).
- Production-grade long-running autonomous agents, multi-agent collaboration, production memory systems, fuller live-ops.
- External game-engine bridges (UE5, Godot, Unity, web clients).
- Marketplace, package signing, dependency resolution (local sharing proof is done; see [`guides/SHARING_DISTRIBUTION.md`](guides/SHARING_DISTRIBUTION.en.md)).
- Final UI visual design, full Studio, ComfyUI-like node editors.
- WASM and remote package execution.

## How to verify this snapshot

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- conformance --list
cargo run -p ygg-cli -- conformance --tag sharing --slowest 3
cargo run -p ygg-cli -- play-create-demo
npm run check --prefix clients/web
npm run build --prefix clients/web
```

If anything fails, the code is the source of truth — update this document.

## Further reading

- [`CHARTER.md`](CHARTER.en.md) — principles that don't change.
- [`architecture/`](architecture/README.en.md) — architecture, kernel, package contract, extension points, events, lifecycles.
- [`product/`](product/README.en.md) — play-creation stance.
- [`protocol/PROTOCOL_V0.md`](protocol/PROTOCOL_V0.en.md) — public protocol.
- [`spec/`](spec/README.en.md) — executable contract matrix and conformance roadmap.
- [`guides/`](guides/README.en.md) — capability-package authoring guides.
- [`roadmap/NEXT_STEPS.md`](roadmap/NEXT_STEPS.en.md) — what's next.
