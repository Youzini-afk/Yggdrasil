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

## Phase S3 — Generic streaming and cancellation lifecycle

Goals:

- Define generic stream frames for capability output.
- Add cancellation/timeout lifecycle records.
- Add fixture/conformance coverage for normal end, error, cancel, and timeout.

Non-goals:

- No model streaming API.
- No agent turn API.

## Phase S4 — SDK/templates and no-network readiness proof

Goals:

- Add TypeScript package-authoring helpers/templates for secret refs, network permission metadata, audit/redaction, and streaming fixtures.
- Add no-network faux model/agent readiness examples that prove the substrate shape without real inference or pi runtime coupling.

Non-goals:

- No real `pi-agent-core` integration yet.
- No real model inference.

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
