# Experience-Led Platform Beta

> [English](./EXPERIENCE_LED_PLATFORM_BETA.en.md) · [中文](./EXPERIENCE_LED_PLATFORM_BETA.md)

This document sets the long-term direction after Agentic Forge Beta: Yggdrasil should converge the foundation-first phase and let real AI-native playable experiences drive the remaining substrate work.

It is not a temporary phase plan and not the design of a single official experience. It is a product-infrastructure strategy: why the current substrate is sufficient to start experience-led work, which infrastructure gaps should be pulled by real experience pressure, what should stay deferred, and how Yggdrasil avoids becoming a chat shell, API gateway, traditional plugin host, or generic agent framework.

## Conclusion

Yggdrasil is not a production-complete platform, but it has enough foundation to stop leading with foundation work.

The stable base now includes:

- A content-free kernel.
- Manifest-driven capability packages.
- No official package privilege, backed by third-party replacement proofs.
- Public protocol, HTTP `/rpc`, SSE, and host stdio.
- Opaque events, SQLite rehydration, assets, branches, projections, and proposals.
- Principals, permission grants, `secret_ref`, outbound executor, redacted audit, and stream/cancel lifecycle.
- Cloud API adapter packages, live model calls, and a transport-neutral inference seam.
- Agentic Forge Beta: package-owned run lifecycle, plan graph, scratch candidates, inference nodes, tool bridge v2, Forge control-room shell, and third-party replacement proof.
- Authoring/composition tooling and hostile conformance.

The next question is no longer:

```text
Can we add one more abstraction layer?
```

It is:

```text
Can the current substrate host a real AI-native experience that is playable, inspectable, modifiable, forkable, and sustainable?
```

If this is not answered by a real experience, more substrate work will become false progress.

## External signals as of 2026-05-20

This section summarizes recent external references and the lessons they imply for Yggdrasil. They are calibration sources, not product shapes to copy, and not dependencies or commitments for Yggdrasil. Source links are included so future work can re-check these signals when external product directions change.

### Roblox Cube / 4D generation: generated objects must enter runtime

Roblox's 2026 Cube / 4D generation direction is a useful signal that generation is moving from static assets toward functional runtime objects: schemas split objects into required parts, geometry is generated, and behavior scripts are retargeted so the generated object can be used by players. References: Roblox, [Accelerating Creation, Powered by Roblox’s Cube Foundation Model](https://about.roblox.com/newsroom/2026/02/accelerating-creation-powered-roblox-cube-foundation-model); Roblox, [Accelerating AI Inference for 3D Creation on Roblox](https://about.roblox.com/newsroom/2025/06/accelerating-ai-inference-roblox-3d-creation).

Implications for Yggdrasil:

- Assets cannot remain a thin opaque put/get/list store forever.
- Generated objects need provenance, derived asset relationships, behavior binding, preview, diff, and runtime attachment metadata.
- In-experience player creation can itself become gameplay, not merely a creator tool.

Yggdrasil should not embed cars, doors, characters, or scenes into the kernel. It should provide strong enough asset/state/proposal/branch substrate for ordinary packages to generate and inspect objects that can enter runtime.

### Roblox Studio agentic: the loop is plan / build / test

Roblox Studio's 2026 agentic direction is a useful signal that mature game-creation agents should focus on planning/building/testing rather than chat: Planning Mode, reviewable/editable action plans, structured task manifests, parallel agents, playtesting agents, log and data-model analysis, self-correcting loops, and third-party tools through unprivileged APIs / MCP. Reference: Roblox, [Roblox Studio is Going Agentic](https://about.roblox.com/newsroom/2026/04/roblox-studio-going-agentic).

Implications for Yggdrasil:

- Agentic Forge Beta is on the right path: an agent should not be a chat box, but a planning, candidate, tool, testing, inspection, and proposal system around creation tasks.
- The next step cannot stop at showing a plan graph. Agents must plan/build/test against a real experience.
- A playtesting agent is valuable because it reads state, logs, events, projections, and user goals, then produces inspectable fixes rather than privileged mutation.

Yggdrasil agents must remain ordinary package-owned creative processes, not `kernel.agent.*`.

### Roblox Hybrid Architecture: do not treat neural world models as game engines

Roblox's 2026 hybrid architecture offers a useful judgment: Video World Models can produce vivid visual dreams, but they lack persistent state, consistent logic, long-term memory, user input control, and true multiplayer simulation. Roblox keeps shared consistent state, symbolic logic, and repeatable simulation in the game engine, while the video model handles stochastic visuals. Reference: Roblox, [Introducing the Roblox Hybrid Architecture](https://about.roblox.com/newsroom/2026/04/roblox-reality-hybrid-architecture-democratizing-photorealistic-multiplayer-gaming).

Yggdrasil principle:

```text
AI may dream; the platform must remember, verify, fork, and recover.
```

Yggdrasil should not chase a pure neural game world. It needs:

- Package-owned state conventions.
- Event-sourced state mutation.
- Snapshot/checkpoint assets.
- Projection-backed inspection.
- Deterministic replay where possible.
- Non-deterministic inference provenance.
- Branch-aware state diff.

These are state substrate, not world ontology. They must not introduce world, scene, character, or turn semantics into the kernel.

### Unity AI: AI must be embedded in creation context and provide data controls

Unity AI's guiding principles are a useful signal that AI tooling must be embedded in creation context while still offering data controls, usage visibility, provider labels, and generated asset metadata. Unity integrates AI into the Editor context, with awareness of project structure, GameObjects, prefabs, render pipeline, console errors, generated asset metadata, usage reporting, provider labels, data ownership, default-off training opt-in, and local Sentis inference. References: Unity, [Unity AI Guiding Principles](https://unity.com/legal/unityai-guiding-principles); Unity, [2026 Unity Game Development Report](https://unity.com/blog/2026-unity-game-development-report-trends).

Implications for Yggdrasil:

- Forge cannot remain only a trace viewer. It must become the creation control room.
- Relevant context includes session, branch, projection, asset graph, package set, proposal queue, agent run, tool/inference trace, failure state, and cost/latency.
- Generated assets need metadata for search, audit, deletion, disclosure, and compliance.
- Data controls and model replacement paths should remain host-owned / package-owned, not platform-hosted key or unified provider administration.

### Inworld Agent Runtime: graph, stream, memory, knowledge, safety, and telemetry are common AI runtime parts

Inworld Runtime is a useful signal that mature AI runtimes often organize around graph / nodes / edges / execution stream, with memory update/retrieve, knowledge, safety, intent, goal advancement, STT/TTS, telemetry, and MCP tool nodes. References: Inworld, [Graphs](https://docs.inworld.ai/node/core-concepts/graphs); Inworld, [Unity Runtime Reference Overview](https://docs.inworld.ai/Unity/runtime/runtime-reference/overview).

Implications for Yggdrasil:

- Agentic Forge's plan graph, stream/cancel, and tool bridge are the right foundation.
- AI-native experiences will soon need ordinary package forms of memory, knowledge, goal/progress, safety, and telemetry capabilities.
- These should be package-owned events/assets/projections/proposals, not kernel ontology.

### Steamworks / GDC: AI games need disclosure, guardrails, cost, and trust strategy

Steamworks separates Pre-Generated AI and Live-Generated AI. Live-Generated AI requires guardrails against illegal content, and ongoing live AI service costs must be managed by the developer. Public GDC 2026 State of the Game Industry summaries show substantial GenAI usage alongside strong negative sentiment in the industry. References: Steamworks, [Content Survey](https://partner.steamgames.com/doc/gettingstarted/contentsurvey); Business Wire / GDC, [2026 State of the Game Industry Report announcement](https://www.businesswire.com/news/home/20260129438528/en/2026-State-of-the-Game-Industry-Report-Reveals-Widening-Effect-of-Layoffs-Broader-Perspectives-on-Generative-AI-Unionization-Tariffs-and-More).

Implications for Yggdrasil:

- AI-generated content needs provenance, metadata, guardrail/audit trails, rights/licensing/disclosure metadata.
- Live AI experiences need cost/usage visibility, policy hooks, redaction, report/export logs.
- Users and creators should be able to understand what AI did, why, from which sources, and with which risks.

Yggdrasil's proposal, approval, audit, redaction, branch, and Forge control-room are strengths, but asset-level disclosure and experience-level observability are still missing.

## Current-stage judgment

### Enough to stop foundation-first

The following is sufficient for a real vertical slice:

- Package loading, capability invocation, explicit provider selection, and no official privilege.
- Public protocol and the Web shell base.
- Minimal play-creation substrate: events, sessions, branches, proposals, projections.
- Secure execution and opt-in live model calls.
- Transport-neutral inference and demoted cloud adapters.
- Agentic Forge's branch-aware candidate / compare / promote / tool / inference scaffold.
- Authoring, composition, and conformance.

Future work should not lead with “complete every substrate first.”

### Thin areas that should be pulled by experience pressure

- Experience runtime contract.
- Package-owned state / snapshot / checkpoint / replay pattern.
- Asset pipeline: content-addressed blobs, provenance graph, derived assets, AI disclosure metadata.
- Memory / knowledge package pattern.
- Experience observability: health, latency, cost, failure breadcrumbs, causal chain.
- Compatibility / migration: package data migration, asset metadata versioning, projection rebuild policy, composition upgrade report.
- Sharing / distribution primitives: composition export/import, branch/session bundle, package-set lockfile.

### Deferred

- Marketplace, creator economy, rating/review, revenue split.
- SaaS billing, user balance, provider key hosting, channel/admin backend.
- Full auth/tenant/cloud product.
- Full realtime multiplayer server, authoritative multiplayer, co-presence conflict resolution.
- Local model manager, weight downloader, GPU scheduler.
- Central moderation product.
- Official world/scene/character/director runtime.

## Principles

### 1. New substrate must be pulled by a real experience

Every new substrate should answer a real experience pressure:

- How does state survive a 20–30 minute session?
- How does the system recover from model failure?
- How does the user understand an agent's change?
- How are generated assets tracked, previewed, deleted, and disclosed?
- How are branches compared?
- How are old sessions migrated after package upgrades?

If new work does not serve these questions, defer it.

### 2. The kernel remains content-free

Do not add:

```text
kernel.agent.*
kernel.model.*
kernel.prompt.*
kernel.memory.*
kernel.world.*
kernel.scene.*
kernel.character.*
kernel.turn.*
kernel.chat.*
```

Acceptable kernel-side work is content-free mechanism: asset blobs, resource policy, projection execution, event subscription permissions, transport parity, health/audit records. Even then, first check whether the existing package/protocol substrate can express it.

### 3. Experiences are package-owned, not kernel-owned

Yggdrasil may define experience package patterns, surface contracts, state snapshot conventions, and checkpoint asset conventions, but it must not own genre or gameplay semantics.

An official reference experience is allowed, but it must be replaceable by third-party packages. Its purpose is to pressure-test the substrate, not define the one true gameplay model.

### 4. Agents are creation collaborators, not privileged executors

Agentic Forge should next plan/build/test against real experiences:

- Observe sessions, branches, projections, assets, and events.
- Explore in scratch branches.
- Produce candidates and comparisons.
- Use tool bridge for plan-only or scoped tool execution proposals.
- Request approval.
- Explain failures and diffs.

Agents must not directly mutate target branches, bypass permission, or hold hidden official privilege.

### 5. AI output must be traceable, explainable, removable, and disclosable

AI-native games cannot mean unrestricted model output. The platform must show:

- Which package / provider / inference / prompt-like input / source refs produced an artifact.
- Whether it is live-generated or pre-generated.
- Whether it passed guardrail / policy / redaction.
- What cost, latency, and failure risk it carried.
- How it can be deleted, replaced, or rolled back from the asset graph.

## Recommended route

### Experience Beta 0 — Thin Experience Runtime Contract

Define how ordinary package-owned experiences run, pause, recover, checkpoint, fork, and receive Agentic Forge changes.

Deliverables:

- Experience package authoring pattern.
- Session-state projection convention.
- Checkpoint asset convention.
- Failure/recovery event shape.
- Play surface state subscription pattern.
- How Forge/Assist connect to an experience session.

Non-goals: `kernel.experience.*`, `kernel.world.*`, `kernel.turn.*`.

### Experience Beta 1 — First Real Playable Vertical Slice

Build an AI-native experience that can be played for 20–30 minutes without waiting for State/Asset/Memory to be complete. It must not be a chat shell, Tavern clone, or prompt/response demo.

Acceptance criteria:

- The user launches it from Home.
- It has package-owned state.
- It uses a real model path, while default conformance remains deterministic/no-network.
- It creates asset/state changes.
- The user asks Assist or Forge for changes.
- Agentic Forge produces plan/candidate/proposal.
- The user can inspect/approve/reject.
- The user can fork a branch.
- The user can compare branches.
- Failures can be recovered, with visible failure breadcrumbs.
- Key asset/proposal/inference provenance is visible.
- A third-party package can replace one key capability.

This is the actual product proof for Yggdrasil. It should start early and pull the minimum required state, asset, memory, and observability work.

### Experience Beta 2 — State + Asset Pipeline Alpha

Make experience state and generated assets trackable, comparable, and recoverable. This phase should deliver only the minimum set exposed by the First Real Playable Vertical Slice; everything else stays as later hardening.

Deliverables:

- Content-addressed asset blobs.
- Asset provenance graph.
- Derived asset refs.
- AI-generated / live-generated metadata.
- Rights/licensing/disclosure metadata slots.
- State snapshot asset.
- State diff preview.
- Branch-aware asset/state views.
- Safe preview descriptors.
- Large output handling.
- Package-scoped asset permission checks.

Non-goals: full media editors, unified media schema, kernel world state model.

### Experience Beta 3 — Experience Observability

Show users and creators what happened, why it failed, and where cost/latency came from. This should start as an acceptance criterion during Experience Beta 1, then become systematic here.

Deliverables:

- Session health.
- Package health.
- Agent run health.
- Model/inference cost and latency summary.
- Proposal causal chain.
- Asset provenance graph view.
- Failure breadcrumbs.
- Stuck run detection.
- Guardrail/audit summary.

Non-goals: full APM, SaaS monitoring backend.

### Experience Beta 4 — Memory / Knowledge Package Alpha

Provide long-term memory and knowledge as ordinary packages, not kernel ontology. If the first real experience needs cross-session / cross-branch long-term memory, build the minimum slice earlier; otherwise this should be pulled after the first vertical slice.

Deliverables:

- Memory record package schema examples.
- Branch-aware memory view.
- Retrieval trace.
- Proposal-gated memory update.
- User correction.
- Forgetting / redaction workflow.
- Memory provenance.
- Knowledge source refs.

Non-goals: `kernel.memory.*`, one official RAG, chat memory system.

### Experience Beta 5 — Creator Loop Beta

Let a new creator build a playable package in a day using docs, templates, and Forge, without reading source code.

Deliverables:

- Better experience templates.
- Fixture runner UX.
- Reload flow polish.
- Composition diagnostics.
- Authoring walkthrough based on a real package.
- Package error explainability.
- Forge authoring workflow.

Non-goals: marketplace, creator monetization.

### Experience Beta 6 — Sharing / Distribution Alpha

Support shareability, reproducibility, and import before marketplace.

Deliverables:

- Export/import composition.
- Export/import branch/session bundle.
- Package-set lockfile.
- Compatibility/migration report.
- AI disclosure metadata bundle.
- Read-only shared session.
- Async fork sharing.

Non-goals: marketplace, package signing network, dependency resolver economy, hosted billing.

## Choosing the first real experience

The first vertical slice can have a small theme, but it must satisfy:

- Not a chat UI.
- Not a Tavern clone.
- No dependency on kernel genre semantics.
- Persistent package-owned state.
- Generated assets or state mutation.
- Agentic Forge participates in creation/modification.
- Branch/fork/compare.
- Failure recovery and provenance.
- The official reference experience must not become a canonical runtime.
- At least one key capability replaceable by a third-party package.

Promising shapes:

- Living sandbox fragment.
- Procedural artifact playground.
- AI-directed scene/workshop.
- Branching worldlet.
- Playable creation board.

The theme is not the point. The point is pressure-testing state, asset, memory, proposal, branch, agent, inference, and Forge.

## Success metrics

Conformance count is no longer the core progress metric. It remains the safety net.

Core metrics become:

- Can a player stay in one experience for 20–30 minutes?
- Can a creator build a playable package in a day?
- Can the user understand why an agent proposed a change?
- Can the user reject a change without polluting the session?
- Can the user fork and compare branch differences?
- Can failures be understood and recovered?
- Are generated assets traceable, previewable, disclosable, and removable?
- Can a third-party package replace an official key capability while preserving the experience?

## Red lines

- Do not put content semantics into the kernel.
- Do not promote cloud API adapters into the platform model abstraction.
- Do not turn Agentic Forge into a chat product or coding-agent clone.
- Do not give official packages hidden privilege.
- Do not let UI read runtime internals or SQLite.
- Do not move marketplace / billing / SaaS key hosting into the mainline prematurely.
- Do not use conformance count as a substitute for real experience validation.

## Direction in one sentence

```text
Yggdrasil has proven that the platform can host free creation;
next it must prove that what it hosts is worth playing, modifying, forking, and returning to.
```
