# Experience-Led Platform Beta Plan

> [English](./EXPERIENCE_LED_PLATFORM_BETA_PLAN.en.md) · [中文](./EXPERIENCE_LED_PLATFORM_BETA_PLAN.md)

This is the execution plan for Experience-Led Platform Beta. The long-term strategy is [`docs/product/EXPERIENCE_LED_PLATFORM_BETA.md`](../product/EXPERIENCE_LED_PLATFORM_BETA.en.md). This is a temporary phase plan and should be deleted once the phases are complete and durable docs are updated.

## Principles

- Shift from foundation-first to experience-led: every new substrate must serve pressure from a real playable experience.
- Keep the kernel content-free: no `kernel.experience.*`, `kernel.world.*`, `kernel.scene.*`, `kernel.character.*`, `kernel.turn.*`, `kernel.agent.*`, `kernel.model.*`, `kernel.memory.*`, or `kernel.chat.*`.
- Experiences are package-owned, not kernel-owned.
- Official reference experiences must be replaceable by third-party packages.
- Default conformance must not require public internet; live model smoke remains explicit opt-in only.
- UI/Forge must use public protocol, events, surfaces, capabilities, assets, projections, and proposals; no runtime internals or SQLite reads.
- Every phase must be validated, committed, and pushed.

## Phase 0 — Strategy lock

Goal: commit the Experience-Led Platform Beta long-term design and synchronized docs.

Deliverables:

- `docs/product/EXPERIENCE_LED_PLATFORM_BETA.md` / `.en.md`.
- README, ALPHA_STATUS, NEXT_STEPS synchronization.
- This plan.

Validation: markdown local link check, `git diff --check`.

## Experience Beta 0 — Thin Experience Runtime Contract

Goal: define how ordinary package-owned experiences start, pause, recover, checkpoint, fork, and bind to Agentic Forge runs.

Deliverables:

- TypeScript SDK: `sdk/typescript/experience-runtime` with package-owned experience descriptors, state projection, checkpoint, recovery, play surface subscription, and Forge/Assist binding types plus self-tests.
- CLI template: `init-package --template experience-runtime` or an extension of `typescript-experience`, producing a deterministic/no-network experience package skeleton.
- Official ordinary reference package: `packages/official/experience-runtime-lab`, only for contract/fixture/capability proof; no world/scene/turn semantics.
- Guide: `docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md` / `.en.md`.
- Conformance: experience contract, checkpoint shape, recovery shape, no kernel experience namespace, basic third-party shape parity.

Red lines: no kernel experience/world/turn methods; no official special routing.

## Experience Beta 1 — First Real Playable Vertical Slice

Goal: build an AI-native reference experience that can be played for 20–30 minutes as early as possible and pulls the next substrate work.

Deliverables:

- Official ordinary package: `packages/official/living-sandbox` or equivalent Ygg-native experience package.
- Package-owned state through opaque events/assets/projections, not kernel ontology.
- Play surface descriptor: launchable from Home and inspectable through Play state/projection.
- Assist/Forge loop: user asks for a change, Agentic Forge produces plan/candidate/proposal, user inspects/approves/rejects, fork/compare works.
- Deterministic/default path: no public internet required for CI/conformance.
- Optional live model path: only through existing inference/model-provider/outbound policy and explicit opt-in.
- Third-party replacement proof: at least one key capability replaceable by an example third-party package.

Validation: workspace tests, conformance, playable vertical CLI demo, package checks, composition check, Web TS.

Red lines: not a chat shell, not a Tavern clone, no canonical game runtime.

## Experience Beta 2 — State + Asset Pipeline Alpha (complete)

Goal: add only the minimum state/asset substrate exposed by the First Real Playable Vertical Slice.

Delivered:

- Stable content-addressed asset helper using FNV-1a 64-bit hash (`fnv1a64:` prefix, deterministic across runs, replaces unstable `DefaultHasher`).
- Standard Beta 2 metadata convention: `content_address`, `provenance`, `disclosure`, `source_refs`, `derived_refs`, `branch_ref`, `state_snapshot_ref`, `projection_ref`, `proposal_ref`, `inference_ref`, `large_output_policy`.
- `official/asset-lab` extended with `content_address` capability (stable content address + metadata convention) and `provenance_graph` capability (asset provenance graph shape with source/derived/disclosure metadata).
- `official/projection-lab` extended with `state_snapshot` capability (state snapshot asset convention and branch-aware diff preview shape).
- `official/playable-creation-board` extended with `preview_state_diff` (branch-aware state diff preview with before/after content addresses) and `describe_asset_provenance` (asset provenance graph with source/derived/disclosure metadata). 13 total capabilities.
- Asset provenance graph: source refs, derived refs, package/provider/inference refs, AI-generated/live-generated/disclosure metadata.
- State snapshot asset convention: checkpoint/recovery/replay hints.
- State/asset diff preview: branch-aware, projection-backed, package-owned.
- Large output handling: tool/model large output through asset refs (existing capability-tool-bridge-lab recommendation reinforced).
- Package-scoped asset permission proof: origin_package_id enforcement, cross-package spoof fail-closed.
- Raw-secret blocking in all Beta 2 capabilities.
- 9 more conformance cases (206 total).

Validation: content address stable, provenance graph, state snapshot convention, state diff preview, playable board metadata, large output asset_ref, package scoped proof — all conformance PASS; `cargo run -p ygg-cli -- playable-board-demo` ok; `cargo run -p ygg-cli -- package check` ok for asset-lab, projection-lab, playable-creation-board.

Red lines: no full media editor, no unified media schema, no state ontology in the kernel — all upheld.

## Experience Beta 3 — Experience Observability (complete)

Goal: show users/creators what happened, why it failed, and where cost/latency came from.

Delivered:

- `official/experience-observability-lab`: package-owned experience observability — session health, package health, agent run health, proposal causal chain, failure breadcrumbs, cost/latency summary, guardrail/audit summary. 8 capabilities, 3 surfaces (forge_panel, assistant_action, home_card). Deterministic, no-network, no inference. All derived from protocol-visible refs, not from SQLite or runtime internals.
- Runtime inproc handler: deterministic/no-network/no inference, outputs public protocol shapes (session_health, package_health, agent_run_health, proposal_causal_chain, failure_breadcrumbs, cost_latency_summary, guardrail_audit_summary). Must not output chat/message/prompt/world/scene/turn/memory shapes.
- Linkage with playable-creation-board: added `summarize_experience_health` capability with observability cross-references.
- Conformance: 10 named cases (contract/session_health/package_health/agent_run_health/proposal_causality/cost_latency/failure_breadcrumbs/guardrail_audit/no_forbidden_namespace/no_raw_secrets).
- Profile autoload: forge-alpha.yaml auto-loads new package.

Validation: `cargo test --workspace`, `cargo run -p ygg-cli -- conformance` (216 cases), `cargo run -p ygg-cli -- package check` for new package.

Red lines: no SaaS APM, no SQLite reads, no kernel.observability.*, no privileged Studio.

## Experience Beta 4 — Memory / Knowledge Package Alpha

Goal: provide long-term memory and knowledge as ordinary packages, minimally according to vertical-slice pressure.

Deliverables:

- `official/memory-lab` or an extension of existing knowledge/context labs, centered on package-owned memory records, retrieval traces, proposal-gated memory updates, forget/redaction, and branch-aware memory views.
- SDK/helper: memory record, retrieval trace, correction, redaction metadata.
- Vertical slice integration if cross-session/branch memory is needed; otherwise readiness proof only.
- Third-party replacement proof.

Validation: memory conformance, raw-secret blocking, branch-aware view, proposal-gated mutation.

Red lines: no `kernel.memory.*`, no one official RAG, no chat memory system.

## Experience Beta 5 — Creator Loop Beta

Goal: let a new creator build a playable package in a day using docs, templates, and Forge, without reading source code.

Deliverables:

- Better experience templates, fixture runner UX, reload flow polish.
- Composition diagnostics for experience package sets, surface slots, replacement candidates, permissions, and state/checkpoint capabilities.
- Forge authoring workflow: package inventory, experience descriptor preview, fixture controls, diagnostics explainability.
- Walkthrough: template to playable package.

Validation: generated package checks, fixture/reload tests, Web TS, doc links.

Red lines: no marketplace/monetization.

## Experience Beta 6 — Sharing / Distribution Alpha + cleanup

Goal: make experiences shareable, reproducible, and importable; delete temporary plans and converge durable docs.

Deliverables:

- Export/import composition bundle.
- Branch/session bundle manifest.
- Package-set lockfile / compatibility report.
- AI disclosure metadata bundle.
- Read-only shared session / async fork sharing proof (local/file-level proof is enough).
- Delete this plan and converge results into README, ALPHA_STATUS, NEXT_STEPS, guides, and product docs.

Validation: export/import conformance, compatibility report tests, doc links, workspace tests, Web TS.

Red lines: no marketplace, package signing network, or hosted billing.
