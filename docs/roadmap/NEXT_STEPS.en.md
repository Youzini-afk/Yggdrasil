# Next Steps

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

The platform foundation is in place. Yggdrasil now has a content-free kernel, manifest-driven packages, real `rust_inproc` and subprocess execution, a permission/principal system, the hook fabric slice, surface contributions, the proposal/approval lifecycle, asset/branch/projection substrate, secure execution primitives, official platform packages, an assistant package, `official/playable-seed`, a blank play-creation loop, and a public-protocol web shell with Home/Play, Forge, Assist, and a bounded text-surface proof.

Agent Infrastructure Alpha, Model Provider Integration Alpha, Live Model Calls Alpha, and Creative Inference Capability Alpha are complete. Yggdrasil can now describe, validate, normalize, and fake/local invoke API differences across OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, and Fireworks as ordinary capability packages; it also has host-owned `secret_ref:env:*`, public `kernel.outbound.execute`, LiveHttpOutboundExecutor, redacted audit, live loopback provider shapes, a transport-neutral inference seam, and inference→proposal proof. Default conformance does not require public internet; manual live smoke must be explicitly opted in. The current headline is **Agentic Forge Beta**: promote agents from lab/proof code into a package-owned, branch-aware, tool-safe, inference-backed, deterministically testable creative agent runtime.

## Where we are

- Platform Foundation Alpha: complete.
- Play/Forge Surface Contract Beta: complete.
- First Real Capability Package Track: complete seed (`composition-lab`, `asset-lab`, `projection-lab`, `playable-seed`; 55 conformance cases).
- Platform Host Alpha: implemented slice complete; remaining items (streaming dispatch, hook timeout audit, persisted provider policy, broader transport parity, richer SDK packaging) are tracked below in Phase I.
- Code Health Split Alpha: complete; CLI commands/templates/conformance, runtime domain behavior, protocol dispatch, and runtime official in-process handlers are split by domain.
- Authoring & Composition Beta+: complete; generated package templates, fixture/reload tooling, composition v2 diagnostics, Forge authoring panels, and a third-party playable replacement proof are in place.
- Secure Execution Substrate: complete Alpha slice. Persistent grants, `secret_ref`, host resolver placeholder, raw-secret blocking, network permission declarations, outbound audit/redaction, generic streaming/cancel lifecycle, secure-execution TypeScript helpers, networked/streaming templates, and no-network model/agent readiness examples are in place.
- Text Surface Proof: complete Phase T1/T2/T3/T4/T5. `integrations/pretext` documents the Pretext reference boundary, and the Assistant Drawer contains a bounded mock streaming text proof over `clients/web/src/text-layout` without kernel/protocol/package changes. `sdk/typescript/text-surface` provides a pure TypeScript frontend SDK for third-party UIs. Font loading, cache diagnostics, and a self-test harness are in place.
- Agent Infrastructure Alpha: complete; `integrations/pi` ledger, `sdk/typescript/ygg-agent-adapter`, `--template agent-runtime`, `official/pi-agent-runtime-lab`, `official/capability-tool-bridge-lab`, Forge/Assist Agent Observability, `thirdparty/agent-runtime` replacement proof, and [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](../guides/AGENT_PACKAGE_AUTHORING.en.md) are in place.
- Model Provider Integration Alpha: complete; `integrations/model-providers` research ledger, `sdk/typescript/model-provider-adapter`, `official/model-provider-lab`, provider profile examples, and [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](../guides/MODEL_PROVIDER_INTEGRATION.en.md) are in place.
- Live Model Calls Alpha: complete; results are folded into [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](../guides/MODEL_PROVIDER_INTEGRATION.en.md), [`docs/ALPHA_STATUS.md`](../ALPHA_STATUS.en.md), and the conformance matrix.
- Creative Inference Capability Alpha: complete; `sdk/typescript/inference-capability` transport-neutral envelope/stream/error/manifest helpers, [`docs/guides/INFERENCE_CAPABILITY_AUTHORING.md`](../guides/INFERENCE_CAPABILITY_AUTHORING.en.md), `official/inference-local-lab` deterministic non-HTTP fake inference provider proof, `official/model-provider-lab` cloud API adapter reposition, and `official/inference-playtest-lab` Ygg-native inference proposal vertical slice are in place. Conformance now has 155 named cases.
- Agentic Forge Beta Phase A: complete; `official/agentic-forge-lab` with describe_contract/start_run/inspect_run/cancel_run/summarize_run/export_plan_graph capabilities, `sdk/typescript/agentic-forge` TS SDK, and 5 conformance cases. Conformance now has 160 named cases.
- Agentic Forge Beta Phase B: complete; extends `official/agentic-forge-lab` with create_candidate/compare_candidate/draft_promote_proposal/archive_candidate/explain_branch_policy capabilities; branch-aware scratch branch intent/metadata; candidate artifacts with stale detection; proposal drafts that never directly mutate target branches; stale target revision mismatch blocks promote; 5 more conformance cases. Conformance now has 165 named cases.
- Agentic Forge Beta Phase C: complete; extends `official/agentic-forge-lab` with run_inference_node/replay_inference_node/validate_inference_output/explain_inference_failure capabilities; 8 explicit plan node kinds; inference providers (deterministic/recorded/cloud_adapter_plan/local_fake); cloud_adapter_plan returns needs_host_policy with no network; replay mismatches flagged never silently passed; inference output action allowlist with forbidden actions; 9-item failure taxonomy with typed recovery hints; 5 more conformance cases. Conformance now has 170 named cases. Phase D is next.

See `docs/ALPHA_STATUS.md` for a detailed snapshot.

## Phase F — Foundation Alpha Consolidation (complete)

Goal: stop expanding surface area. Sand the rough edges, lock the contract, and make the existing foundation easy to demo, document, and extend.

- Documentation refresh across `README.md`, `README.md`, and the docs tree.
- Add `docs/product/PLAY_CREATION_MODEL.md` to fix the play-creation product stance.
- Add `docs/ALPHA_STATUS.md` as the living snapshot of what is done, partial, and deferred.
- Resolve remaining Platform Host Alpha partial items where they are cheap.
- @oracle-led review pass for content-shape leaks, official-privilege leaks, and YAGNI cleanups.
- A single canonical end-to-end demo path documented and validated through conformance.

This phase finishes when a new contributor can clone the repo, read one README, run one host serve command, and reach the blank play-creation loop without surprises.

## Phase G — Playable Experience Alpha seed (complete)

Goal: prove the substrate by building reference packages that are launchable, inspectable, forkable, and assistant-assistable, entirely as ordinary packages.

This is the first time the platform produces something a player-creator can sit with for more than a demo. It is not a SillyTavern, not a chat-only runtime, not a director — it is the smallest experience that exercises every substrate primitive honestly.

This seed is intentionally not a canonical game runtime. `official/playable-seed` proves the package path; `official/composition-lab`, `official/asset-lab`, and `official/projection-lab` prove the surrounding authoring and inspection loops.

Constraints carried into this phase:

- Kernel changes are last resort. If the experience needs a new primitive, redesign the experience first.
- The official package implementing the experience must remain replaceable by a third-party package.
- The assistant must propose changes through `kernel.proposal.*`, not through privileged paths.
- Forge must be able to inspect, fork, and edit the experience using only the public protocol.
- Conformance grows alongside the package: at least one hostile case proves third-party experience packages reach the same surfaces.

## Phase H — Authoring & Composition Beta+ (complete)

Goal: turn the current authoring slice (`init-package`, `init-composition`, `composition check`, generated experience template) into a real authoring loop someone outside this repo can use to ship a package.

- Template variants per surface slot (`basic`, `experience`, `play-renderer`, `forge-panel`, `assistant-action`, `asset-editor`, `full-surface`).
- Local fixture and reload tooling: `package check`, `package run-fixture`, `package reload`, and generated package conformance.
- Composition descriptor v2 diagnostics for optional packages, required capabilities, permission expectations, replacement candidates, and compatibility notes.
- Forge authoring surface improvements — package/capability inventory, surface descriptor inventory by slot, composition diagnostics, and manifest/template CLI guidance.
- Third-party replacement proof: `examples/packages/thirdparty-playable-seed` and `examples/compositions/playable-seed-replacement` prove official packages are replaceable without official priority.
- Durable walkthrough updates in `docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`.

## Phase I — Secure execution and host hardening (background)

Carried forward as background work, not the headline:

- Richer resource policy coverage beyond network declarations, especially filesystem and package-principal asset/projection permissions.
- Content-addressed asset blobs.
- Package-owned projection execution.
- Package-principal subscribe permissions and broader stream transport parity.
- Hook handler timeout/error audit.
- Persisted capability provider selection policy.
- Broader transport parity coverage in conformance.
- WASM and remote package entry execution.

These items unblock specific use cases. They do not block Agent Infrastructure Alpha, but every agent/model package must use the existing public protocol, permission, audit, redaction, streaming, and proposal paths.

## Phase J — Agent Infrastructure Alpha (complete)

Goal: make Yggdrasil able to host, constrain, observe, and replace agent-like packages while keeping agent semantics outside the kernel.

Delivered:

- `docs/architecture/PI_INTEGRATION.md` and `integrations/pi` ledger fix the pi absorption boundary.
- `sdk/typescript/ygg-agent-adapter` maps Yggdrasil capabilities to pi-style tools through public protocol calls; no private runtime access.
- `--template agent-runtime` generates deterministic/no-network agent-like packages with package-owned traces and approval-gated proposals.
- `official/pi-agent-runtime-lab` is an ordinary reference package with no special routing, no hidden permissions, and no real model calls.
- `official/capability-tool-bridge-lab` discovers capabilities, previews permissions, requires explicit provider selection, and only builds `kernel.capability.invoke` / `kernel.capability.stream` plans, avoiding confused deputy behavior.
- Forge/Assist show agent traces, tool diagnostics, and readiness badges using package-owned events, proposals, surfaces, and public protocol.
- `examples/packages/thirdparty-agent-runtime` and `examples/compositions/agent-runtime-replacement` prove official agent packages are not privileged.
- `docs/guides/AGENT_PACKAGE_AUTHORING.md` is the durable authoring guide.

Non-goals for Phase J:

- No real model inference until a dedicated package uses the secure execution substrate and explicit host policy.
- No kernel `agent`, `prompt`, `memory`, `turn`, or `model` methods.
- No wholesale embedding of `pi-coding-agent` product assumptions.

## Phase K — Model Provider Integration Alpha (complete)

Goal: start real model provider integration directly while keeping the Yggdrasil shape: ordinary packages, `secret_ref`, network allowlists, redacted audit, stream/cancel, fake/local conformance, manual live opt-in, no official privilege, and no kernel model ontology.

Delivered: provider API research ledger (M0), `sdk/typescript/model-provider-adapter` (M1), `official/model-provider-lab` no-network normalization (M2), host outbound executor boundary (M3), OpenAI/Anthropic/Gemini invoke adapters (M4), OpenAI-compatible/OpenRouter/DeepSeek/xAI/Fireworks presets (M5), streaming normalization (M6), provider profile examples, durable guide, and 114 conformance cases.

Non-goals: user balances, billing, channel admin, admin UI, hosted platform relay keys, `kernel.model.*`, `kernel.prompt.*`, `kernel.chat.*`, and `kernel.embedding.*`.

## Phase L — Live Model Calls Alpha (complete)

Goal: advance the fake/local provider path into real live calls while still using ordinary capability packages, host-owned secrets, public outbound boundaries, redacted audit, and opt-in live conformance.

Delivered: L0 live-call contract, L1 `EnvSecretResolver`, L2 `LiveHttpOutboundExecutor` (`reqwest + rustls`, disabled by default), L3 public `kernel.outbound.execute`, L4 DeepSeek canary / secret header injection / loopback live HTTP, L5 OpenAI / Anthropic / Gemini live adapter shapes, L6 OpenRouter / DeepSeek / xAI / Fireworks quirks and sanitized fixtures, and L7 durable docs cleanup. Current conformance has 145 named cases.

Non-goals: relay gateway, user balances/billing, channel admin, hosted platform relay keys, default networked CI, provider direct env access, provider direct HTTP bypassing the host, and `kernel.model.*`.

## Phase M — Creative Inference Capability Alpha (complete)

Goal: keep Yggdrasil's near-term product path cloud API first, but prevent the platform abstraction from becoming cloud API shaped. Cloud API adapters are ordinary packages, not the Ygg model abstraction. This phase proves a transport-neutral inference capability seam, a non-HTTP fake provider, and an inference → proposal/inspection/branch/fork creative runtime loop.

- C0: API-first but not API-shaped ADR and temporary plan (complete).
- C1: transport-neutral inference capability contract (complete; `sdk/typescript/inference-capability` + `docs/guides/INFERENCE_CAPABILITY_AUTHORING.md`).
- C2: non-HTTP fake local provider proof (complete; `official/inference-local-lab` + 5 conformance cases).
- C3: cloud adapter package reposition (complete; `official/model-provider-lab` is a cloud adapter, not the platform abstraction).
- C4: Ygg-native inference proposal vertical slice (complete; `official/inference-playtest-lab` + 5 conformance cases).
- C5: durable docs cleanup (complete; temporary plan removed, durable content folded into guides/status/next steps).

Non-goals: local LLM platform, weight/GPU/scheduling system, more provider zoo expansion, unified chat schema, API gateway, and `kernel.model.*`.

## Phase N — Agentic Forge Beta (in progress)

Goal: promote Agent Infrastructure Alpha from safe-hosting proof into a Yggdrasil-native creative agent runtime. Agentic Forge agents are ordinary package-owned creative processes: they maintain run lifecycle, working state, plan graph, and candidates; explore in scratch branches by default; interact with target branches through candidate compare / proposal / inspection / approval / promote; use scoped grants and audit for tool calls; support replaceable live inference and deterministic fallback; and present run timelines, plan graphs, scratch diffs, candidate comparison, and tool/inference traces in Forge instead of chat transcripts.

Stages are tracked in [`AGENTIC_FORGE_BETA.md`](AGENTIC_FORGE_BETA.en.md):

- Phase 0: plan and boundary lock (in progress).
- Phase A: package-owned run lifecycle / working state / plan graph.
- Phase B: branch-aware scratch branch / candidate / compare / promote proof.
- Phase C: inference-backed agent run with deterministic fallback.
- Phase D: tool bridge v2 scoped toolchain observation / risk / replay.
- Phase E: Forge Agent Workspace / Observability UI shell.
- Phase F: third-party replacement proof, hostile conformance, durable docs cleanup.

Non-goals: LangChain clone, chat shell, coding-agent clone, agent marketplace, always-on autonomous background agents, provider zoo, OpenAI-compatible agent endpoint, `kernel.agent.*` / `kernel.model.*` / `kernel.prompt.*` / `kernel.memory.*`.

## Deferred indefinitely from kernel scope

These remain non-goals for the kernel. They may exist as future packages.

- SillyTavern compatibility — see `docs/tavern/TAVERN_COMPAT.md`.
- pi product embedding — see `docs/architecture/PI_INTEGRATION.md`. Agent infrastructure may proceed only as ordinary package/SDK work.
- External game engine bridges (UE5/Godot/Unity, web clients).
- Any UI shell, inspector, or studio beyond the public-protocol web shell skeleton.
- Memory model, world simulation, director, prompt rendering, and model provider abstraction in the kernel. Agent loops, production-grade live model calls, and model providers may exist only as ordinary packages.
- Marketplace, package signing, dependency resolver.

## How to read this list

Phase F, the seed form of Phase G, Creative Capability Kit Alpha, Model Connectivity Kit Alpha, Code Health Split Alpha, Runtime Split Alpha, Authoring & Composition Beta+, Secure Execution Substrate Alpha, Optional Text Engine Alpha, Agent Infrastructure Alpha, Model Provider Integration Alpha, Live Model Calls Alpha, and Creative Inference Capability Alpha are complete. Agentic Forge Beta is in progress. Every next phase is graded on charter discipline: no content shapes leaking into the kernel, no official privilege leaking through any path, and all package/UI behavior using public protocol boundaries.
