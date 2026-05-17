# Secure Execution Substrate Alpha

> [English](./SECURE_EXECUTION_SUBSTRATE_ALPHA.md) · [中文](./SECURE_EXECUTION_SUBSTRATE_ALPHA.zh-CN.md)

This temporary execution plan combines two threads:

1. **Secure execution substrate** — the generic safety/runtime contracts required before real model inference, pi agent packages, Tavern bridges, remote packages, or other networked/streaming/side-effect packages.
2. **Text Surface Proof** — a small frontend proof inspired by Pretext for future streaming agent/model text output. This remains UI infrastructure, not a kernel feature.

This plan is intentionally substrate-first. It does not add model, prompt, agent, chat, memory, Tavern, or director concepts to the kernel.

## Invariants

- The kernel stays content-free.
- Secret references, network permissions, audit envelopes, redaction states, streams, and cancellation are generic execution contracts.
- Provider/model/agent/Tavern semantics remain package-owned.
- Official packages receive no special permission or routing priority.
- UI proofs use public protocol/client-side infrastructure only.

## Phase S1 — Persistent permissions and secret references ✅

Goals:

- Persist scoped permission grants through the event log so host restarts can rehydrate grants. **Done.**
- Add a generic `secret_ref` contract and host resolver placeholder. **Done.**
- Add hostile conformance for durable grants and raw-secret blocking in known trusted paths. **Done.**

Non-goals:

- No production secret vault. **Met: `DenyAllSecretResolver` is the default.**
- No provider-specific key handling. **Met.**
- No real network/model calls. **Met.**

## Phase S2 — Network permissions, outbound audit, and redaction skeleton ✅

Goals:

- Extend manifest permission metadata with network declarations. **Done.**
- Add generic outbound audit/redaction records and helpers. **Done.**
- Add no-network/allowlisted-network conformance fixtures through package capabilities or host helpers. **Done.**

Non-goals:

- No claim of full OS-level subprocess sandboxing. **Met.**
- No provider-specific audit schema. **Met.**

## Phase S3 — Generic streaming and cancellation lifecycle ✅

Goals:

- Define generic stream frames for capability output. **Done.**
- Add cancellation/timeout lifecycle records. **Done.**
- Add fixture/conformance coverage for normal end, error, cancel, and timeout. **Done.**

Non-goals:

- No model streaming API. **Met.**
- No agent turn API. **Met.**

## Phase S4 — SDK/templates and no-network readiness proof ✅

Goals:

- Add TypeScript package-authoring helpers/templates for secret refs, network permission metadata, audit/redaction, and streaming fixtures. **Done.**
- Add no-network faux model/agent readiness examples that prove the substrate shape without real inference or pi runtime coupling. **Done.**

Deliverables:

- `sdk/typescript/secure-execution/index.ts`: Secret reference construction/validation (`secretRef`, `isValidSecretRef`, `looksLikeRawSecret`, `isSecretFieldName`), network declaration helper (`NetworkDeclaration` class with manifest entry and host/method matching), outbound audit/redaction helper (`OutboundAuditHelper` with audit-safe request payload builder that rejects raw secrets), and stream frame client (`StreamFrameClient` with full start/chunk/progress/end/error/cancel/timeout lifecycle). All helpers wrap only public protocol and types — no private internals, no protocol bypass.
- `--template networked`: Generates a subprocess package with network permission declarations (`host`, `methods`, `purpose`), a `fetch` capability with `network` side effect, and an `echo` capability. The TypeScript template imports secure-execution helpers and demonstrates `secretRef`, `NetworkDeclaration`, and `OutboundAuditHelper` usage. Manifest includes `permissions.network.declarations`. No raw secrets, no implicit network access.
- `--template streaming`: Generates a subprocess package with a streaming capability (`streaming: true`) and an echo capability. The TypeScript template imports `StreamFrameClient` and demonstrates faux streaming frame lifecycle (start, chunk, end). No real model inference.
- `examples/packages/faux-model-readiness/`: No-network readiness proof for model-like capability packages. Declares network permissions, provides `discover` and `stream-faux` capabilities, uses `secret_ref` for credentials, returns discovery plans (not real API responses), produces faux streaming frames. No real inference or network calls.
- `examples/packages/faux-agent-readiness/`: No-network readiness proof for agent-like capability packages. Provides `propose` and `stream-trace` capabilities, produces proposals/traces/plans only (no real agent loop), emphasizes public protocol/capability/proposal patterns, no network permissions needed. No connection to pi runtime or model inference.
- Conformance: 5 new cases covering generated networked template, generated streaming template, faux-model-readiness manifest structure, and faux-agent-readiness manifest structure. All verify no raw secrets, proper network declarations, streaming capabilities, and substrate shape.

Non-goals:

- No real `pi-agent-core` integration yet. **Met.**
- No real model inference. **Met.**

## Phase T1 — Pretext-inspired text surface proof

Goals:

- Add an `integrations/pretext` ledger documenting what Pretext is useful for and what it is not.
- Add a lightweight client-side text layout/progressive streaming proof in the Assistant drawer or a contained web module.
- Keep existing Play/Forge dashboards stable; do not rewrite the whole web shell.

Non-goals:

- No kernel/package/protocol changes for Pretext.
- No full Markdown engine commitment.
- No dependency commitment until the proof shows value.

## Final phase — durable docs and cleanup

Goals:

- Update durable docs/status/conformance matrix.
- Remove this temporary plan document after the milestone completes.
- Run full validation.

Required checks:

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

Also run representative package/composition checks and doc-link validation.
