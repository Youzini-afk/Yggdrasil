# Agent Infrastructure Alpha

> [English](./AGENT_INFRASTRUCTURE_ALPHA.en.md) · [中文](./AGENT_INFRASTRUCTURE_ALPHA.md)

This is a temporary execution plan. Delete it after completion and fold durable results into the README, `docs/ALPHA_STATUS.md`, `docs/roadmap/NEXT_STEPS.md`, conformance matrix, and durable guides.

Goal: make Yggdrasil able to host, constrain, observe, and replace agent-like capability packages without adding agent/model/prompt/memory/turn semantics to the kernel.

## J0 — PI Integration Ledger ✅

- Upgrade `docs/architecture/PI_INTEGRATION.md` / `.en.md`.
- Add `integrations/pi` ledger, lock, and capability map.
- Fix pi absorption tiers: adapter-now, package-internal optional, reference-only, deferred, rejected.

## J1 — Ygg Agent Adapter SDK

- Add `sdk/typescript/ygg-agent-adapter`.
- Provide capability-to-tool, tool invoke/stream, proposal helper, trace helper, permission/provider diagnostics.
- Do not import private runtime or depend on `pi-coding-agent`.

## J2 — Agent Runtime Template

- Add `ygg init-package --template agent-runtime`.
- Default deterministic/no-network subprocess package.
- Include streaming run capability, assistant_action/forge_panel surfaces, proposal-first output, package-owned trace events.

## J3 — Official Reference Agent Package

- Add ordinary `packages/official/pi-agent-runtime-lab` package.
- no-network/faux by default, no real model calls.
- Can stream runs, draft proposals, and emit trace.
- No official privilege or special routing.

## J4 — Capability Tool Bridge Lab

- Add an ordinary tool bridge package.
- Discover capabilities, preview permissions, require explicit provider selection, call through `kernel.capability.invoke/stream`.
- Hostile conformance covers ambiguous provider, denied invoke, official no-priority.

## J5 — Forge / Assist Observability

- Show agent trace, tool timeline, proposal explanation, stream text, audit/redaction badges.
- Use only public protocol and surface discovery.

## J6 — Third-party Replacement Proof

- Add third-party agent runtime example and composition replacement.
- Prove third-party and official agents can reach the same surface/capability/proposal/trace paths.

## J7 — Durable Docs + Cleanup

- Update README, ALPHA_STATUS, NEXT_STEPS, CONFORMANCE_MATRIX, package authoring guide.
- Add agent package authoring guide.
- Delete this temporary plan.

## Non-goals

- No `kernel.agent.*`, `kernel.model.*`, `kernel.prompt.*`, `kernel.memory.*`, or `kernel.turn.*`.
- No real model inference.
- No wholesale `pi-coding-agent` embedding.
- No default bash/read/write/edit tools.
- No priority for official agent packages.
