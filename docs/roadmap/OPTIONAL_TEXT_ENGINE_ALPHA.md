# Optional Text Engine Alpha

> [English](./OPTIONAL_TEXT_ENGINE_ALPHA.md) · [中文](./OPTIONAL_TEXT_ENGINE_ALPHA.zh-CN.md)

This temporary execution plan turns the current Text Surface Proof into an optional frontend text engine track. Pretext is treated as an optional client-side layout engine, not a kernel feature and not an official capability package.

## Invariants

- No `kernel.text.*`, `kernel.model.*`, `kernel.agent.*`, or `kernel.prompt.*` methods.
- No `official/pretext-*` package.
- Fallback text layout remains always available.
- Pretext, if used, is behind a Web client engine abstraction and dynamic selection.
- Assistant/Forge/Play consume generic stream/text surfaces, not model/agent semantics.

## Phase T2 — Engine abstraction and fallback registry

Goals:

- Introduce a `TextEngine` interface, engine registry, config, and fallback engine implementation.
- Refactor the existing text-layout adapter so current Assistant proof behavior is preserved.
- Add stream-frame-to-text-buffer adapter helpers for generic stream frames.

Validation:

- `tsc -p clients/web/tsconfig.json --noEmit`
- Existing Rust/conformance checks unaffected.

## Phase T3 — Optional Pretext engine and feature flags

Goals:

- Add optional `PretextEngine` behind dynamic import / runtime engine selection.
- Keep the repo buildable without installing Pretext.
- Add runtime controls via URL/localStorage/build environment fallbacks.
- Update `integrations/pretext` ledger and client README.

Validation:

- Fallback works when Pretext is unavailable.
- Engine selection diagnostics are visible in the Assistant proof.

## Phase T4 — Forge/Assistant stream text integration

Goals:

- Wire the text buffer adapter to generic stream frame shapes.
- Add a bounded Forge text preview for stream/proposal/tool/audit-like long text without replacing JSON inspectors.
- Keep Play unchanged except for documented future optional hint design.

Validation:

- Web TypeScript passes.
- UI behavior remains public-protocol-only.

## Phase T5 — SDK extraction, tests, and hardening

Goals:

- Extract reusable text-surface helpers under `sdk/typescript/text-surface`.
- Add lightweight unit tests for fallback engine, registry, stream adapter, and engine selection.
- Add cache limits and font-loading helpers.
- Document third-party client usage.

Validation:

- TypeScript tests pass.
- Existing Rust/conformance/play demo pass.

## Final phase — durable docs and cleanup

Goals:

- Update durable docs/status/roadmap.
- Delete this temporary plan after completion.
- Run full validation.

Required checks:

```bash
tsc -p clients/web/tsconfig.json --noEmit
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
```
