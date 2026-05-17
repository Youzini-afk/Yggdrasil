# Runtime Split Alpha

> [English](./RUNTIME_SPLIT_ALPHA.md) · [中文](./RUNTIME_SPLIT_ALPHA.zh-CN.md)

Runtime Split Alpha is a code-health and contract-hardening track. It prevents `crates/ygg-runtime/src/runtime.rs` and the official in-process package fallback from becoming long-term architecture traps.

This track is not a feature expansion. It preserves the public `Runtime<S>` API and the current package/protocol model, while deliberately fixing two unsafe patterns: protocol registry/dispatch drift and suffix-only in-process fallback routing.

## Goals

- Keep `Runtime<S>` public methods stable.
- Split runtime behavior by kernel domain instead of by ad-hoc helper bucket.
- Make protocol methods a single source of truth for registry and dispatch.
- Ensure implemented protocol methods cannot silently lack dispatch coverage.
- Ensure in-process official handlers route by provider package and declared capability, not suffix alone.
- Keep the kernel content-free and official packages unprivileged.

## Non-goals

- No asset store redesign.
- No projection engine redesign.
- No new gameplay/content/model semantics.
- No new direct service routes.
- No trait-heavy service layer or second runtime implementation.
- No package dependency resolver, WASM execution, remote execution, or marketplace work.

## Phase A — Protocol single source of truth

Create a `KernelMethod` source of truth that owns method id, status, streaming flag, and parsing. Derive protocol registry metadata and dispatch matching from it.

Acceptance:

- `kernel.session.close` is dispatch-covered or status-corrected; prefer dispatch because runtime already supports close.
- Methods dispatched by `Runtime::call_protocol` are represented in the registry.
- Implemented/partial method coverage is tested.
- Public method names do not change.

## Phase B — Split `runtime.rs` by kernel domain

Keep `runtime.rs` as the stable module root and move impl blocks/types into domain modules:

- `session.rs`
- `events.rs`
- `packages.rs`
- `capabilities.rs`
- `hooks.rs`
- `permissions.rs`
- `assets.rs`
- `branches.rs`
- `projections.rs`
- `proposals.rs`
- `protocol_dispatch.rs`

Acceptance:

- Existing `Runtime<S>` public methods still compile for callers.
- Moved public request/record types remain re-exported from `ygg_runtime::runtime`.
- `runtime.rs` becomes a table of contents, not a coordinator blob.
- Runtime unit tests pass after the split.

## Phase C — Harden in-process official fallback routing

Replace suffix-only fallback behavior in `inproc/common.rs` with package-aware routing.

Acceptance:

- A handler is selected by provider package id plus declared local capability name.
- Unknown or unimplemented registered in-process capabilities fail loudly instead of returning generic success.
- Conformance proves unrelated packages do not receive official fallback behavior by suffix.
- Existing official labs still pass.

## Phase D — Documentation and final validation

Update status and roadmap docs, then run full validation.

Required checks:

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

Also run package checks for representative official labs and a doc-link check.

## Invariants

- Runtime remains a content-free platform kernel.
- Official packages do not receive prefix-based privilege.
- Protocol methods have one registry status and one dispatch decision.
- No `RwLock` guard should be held across `.await` unless explicitly reviewed.
- Error text should not change accidentally because protocol error classification currently depends on message contents.
