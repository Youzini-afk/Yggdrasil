# Agent Infrastructure Alpha

> [English](./AGENT_INFRASTRUCTURE_ALPHA.en.md) · [中文](./AGENT_INFRASTRUCTURE_ALPHA.md)

This is a temporary execution plan. Delete it after completion and fold durable results into the README, `docs/ALPHA_STATUS.md`, `docs/roadmap/NEXT_STEPS.md`, conformance matrix, and durable guides.

Goal: make Yggdrasil able to host, constrain, observe, and replace agent-like capability packages without adding agent/model/prompt/memory/turn semantics to the kernel.

## J0 — PI Integration Ledger ✅

- Upgrade `docs/architecture/PI_INTEGRATION.md` / `.en.md`.
- Add `integrations/pi` ledger, lock, and capability map.
- Fix pi absorption tiers: adapter-now, package-internal optional, reference-only, deferred, rejected.

## J1 — Ygg Agent Adapter SDK ✅

- Add `sdk/typescript/ygg-agent-adapter`.
- Provide capability-to-tool, tool invoke/stream, proposal helper, trace helper, permission/provider diagnostics.
- Do not import private runtime or depend on `pi-coding-agent`.
- **Deliverables**:
  - `sdk/typescript/ygg-agent-adapter/index.ts` — pure TS, no external dependencies. Contains ProtocolClient interface; stable types (CapabilityDescriptor / CapabilityTool / ToolCall / ToolResult / AgentTraceEvent / AgentProposalDraft); createYggAgentAdapter factory; capabilityToTool / createCapabilityTool / invokeCapabilityTool / streamCapabilityTool; createTraceEvent / createProposalDraft; diagnosePermissions / diagnoseProvider / blockRawSecrets; runYggAgentAdapterSelfTest covering tool mapping, ambiguous provider rejection, proposal draft, trace event, stream request, raw secret blocking.
  - `sdk/typescript/ygg-agent-adapter/README.md` / `README.en.md` — Chinese & English docs.
  - `integrations/pi/capability-map.yaml` — J1 adapter SDK annotation.

## J2 — Agent Runtime Template ✅

- Add `ygg init-package --template agent-runtime`.
- Default deterministic/no-network subprocess package.
- Include streaming run capability, assistant_action/forge_panel surfaces, proposal-first output, package-owned trace events.
- **Deliverables**:
  - `crates/ygg-cli` — `PackageTemplate::AgentRuntime`, `EffectiveTemplate::AgentRuntime`, manifest generation (4 capabilities: run streaming, explain-run, draft-proposal, echo; 2 surfaces: assistant_action + forge_panel; permissions: {}).
  - `crates/ygg-cli/src/templates/mod.rs` — `typescript_agent_runtime_template()`; uses `StreamFrameClient` (secure-execution) and `createTraceEvent`/`createProposalDraft`/`blockRawSecrets` (ygg-agent-adapter).
  - `crates/ygg-cli/src/conformance/generated.rs` — `generated_agent_runtime_template()` conformance case: verifies 4 capabilities, run streaming, assistant_action + forge_panel surfaces, no-network, no raw secrets, no kernel.agent/model/prompt/memory/turn text.
  - Conformance total +1 (99 named cases).

## J3 — Official Reference Agent Package ✅

- Add ordinary `packages/official/pi-agent-runtime-lab` package.
- no-network/faux by default, no real model calls.
- Can stream runs, draft proposals, and emit trace.
- No official privilege or special routing.
- **Deliverables**:
  - `packages/official/pi-agent-runtime-lab/manifest.yaml` — ordinary package, 5 capabilities (run streaming, explain_run, draft_proposal, summarize_trace, echo), 3 surfaces (assistant_action + forge_panel + home_card), approval_policy fork_then_approve, permissions {} with no network declarations.
  - `crates/ygg-runtime/src/inproc/pi_agent_runtime_lab.rs` — inproc handler returning deterministic/no-network/faux payloads (pi_agent_run_plan, pi_agent_run_explanation, pi_agent_proposal, pi_agent_trace_summary, pi_agent_echo) with provenance containing provider_package_id.
  - `crates/ygg-cli/src/conformance/official_labs.rs` — `pi_agent_runtime_lab()` conformance case: verifies no-inference/no-network, approval-gated proposal, surfaces discoverable, provider_package_id match.
  - Conformance total +1 (100 named cases).

## J4 — Capability Tool Bridge Lab ✅

- Add an ordinary tool bridge package.
- Discover capabilities, preview permissions, require explicit provider selection, call through `kernel.capability.invoke/stream`.
- Hostile conformance covers ambiguous provider, denied invoke, official no-priority.
- **Deliverables**:
  - `packages/official/capability-tool-bridge-lab/manifest.yaml` — ordinary package, 6 capabilities (discover_tools, preview_tool_permissions, invoke_tool, stream_tool, explain_tool_call, echo), 3 surfaces (forge_panel + assistant_action + home_card), permissions {} with no network declarations.
  - `crates/ygg-runtime/src/inproc/capability_tool_bridge_lab.rs` — inproc handler returning deterministic tool-bridge plans (discover_tools marks ambiguous/rejected without preferring official; preview_tool_permissions reports missing_permissions; invoke_tool/stream_tool require explicit provider, ambiguous/missing rejected; explain_tool_call returns audit-safe summary; raw secret payload returns unsafe_blocked).
  - `crates/ygg-cli/src/conformance/official_labs.rs` — `capability_tool_bridge_lab()` conformance case: verifies load package; discover_tools marks ambiguous providers rejected; explicit third-party provider works as plan; official provider not preferred; invoke_tool missing provider rejected; preview denied reports missing permission; raw secret payload unsafe_blocked; surfaces discoverable.
  - Conformance total +1 (101 named cases).

## J5 — Forge / Assist Observability ✅

- Show agent trace, tool timeline, proposal explanation, stream text, audit/redaction badges.
- Use only public protocol and surface discovery.
- **Deliverables**:
  - `clients/web/src/agent/observability.ts` — pure UI helper that extracts agent-like observability from events/proposals/surfaces/capabilities using generic string heuristics (no hardcoded official package).
  - `clients/web/src/surfaces/forge.ts` — adds "Agent Observability" section: cards/summary (agent pkg/surf/run/tool/proposal/stream counts), trace timeline (latest package-owned trace/tool/run signals), tool bridge diagnostics badges (ambiguous/rejected/provider/permission/redaction), proposal explanation (reuses T4 text preview). Existing JSON inspectors preserved.
  - `clients/web/src/drawer/assistant.ts` — adds lightweight "Agent Readiness" panel: counts of discovered agent-like surfaces/capabilities, emphasizing no real model / no network / proposal-gated / tool bridge plan-only. Buttons disabled; no real agent launch.
  - `clients/web/src/main.ts` — passes auxiliary view data into Forge and Assistant Drawer.
  - `clients/web/src/styles.css` — Agent Observability and Agent Readiness styles.
  - `clients/web/README.md` — J5 docs.

## J6 — Third-party Replacement Proof ✅

- Add third-party agent runtime example and composition replacement.
- Prove third-party and official agents can reach the same surface/capability/proposal/trace paths.
- **Deliverables**:
  - `examples/packages/thirdparty-agent-runtime/manifest.yaml` — ordinary third-party package, 5 capabilities (run streaming, explain_run, draft_proposal, summarize_trace, echo), 3 surfaces (assistant_action + forge_panel + home_card), permissions {} with no network declarations.
  - `crates/ygg-runtime/src/inproc/thirdparty_agent_runtime.rs` — inproc handler returning deterministic/no-network/faux payloads (thirdparty_agent_run_plan, thirdparty_agent_run_explanation, thirdparty_agent_proposal, thirdparty_agent_trace_summary, thirdparty_agent_echo) with provenance containing provider_package_id.
  - `examples/compositions/agent-runtime-replacement/composition.yaml` — required package points to third-party agent runtime, replacement_candidates includes official/pi-agent-runtime-lab, required_capabilities covers run/draft/explain/summarize, permission_expectations capabilities.invoke, compatibility notes explain official no-priority.
  - `crates/ygg-cli/src/conformance/replacement.rs` — `thirdparty_agent_runtime_surfaces()`, `thirdparty_agent_runtime_invocation()`, `composition_agent_runtime_replacement()` conformance cases: verifies surfaces discoverable, no-inference/no-network, approval-gated proposal, provenance match, composition check passes.
  - Conformance total +3 (104 named cases).

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
