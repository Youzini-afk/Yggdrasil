# Authoring & Composition Beta+

> [English](./AUTHORING_COMPOSITION_BETA.md) · [中文](./AUTHORING_COMPOSITION_BETA.zh-CN.md)

Authoring & Composition Beta+ is the next platform proving track after the foundation, official labs, code-health split, and runtime split. TavernHeadless has already informed the first creative/model capability packages; this track shifts the center of gravity to third-party package authorship and replaceable compositions.

The goal is not to build a Tavern clone, a full Studio, or a game runtime. The goal is to make the ordinary package path real enough that an external author can create, run, inspect, compose, and replace packages through the same public protocol used by official packages.

## Goals

- Make `ygg init-package` useful for multiple surface-slot package shapes.
- Add local package fixture execution and diagnostics that do not depend on private runtime access.
- Make package reload/restart/log flows practical for development.
- Strengthen composition descriptors so package sets can be checked, launched, and replaced explicitly.
- Improve Forge as a public-protocol package/composition authoring surface.
- Prove that a third-party playable package can replace an official seed without official priority or kernel hardcoding.

## Non-goals

- No chat/message/turn runtime.
- No live model inference.
- No SillyTavern compatibility runtime.
- No marketplace, package signing, dependency resolver, or package registry service.
- No private Forge backdoors into runtime internals.
- No official package privilege.

## Phase H1 — Authoring templates and fixture runner

Expand generated package templates beyond the current single experience template.

Deliverables:

- Template variants for `experience_entry`, `play_renderer`, `forge_panel`, `assistant_action`, and `asset_editor` surfaces.
- A local fixture runner that invokes declared capabilities with canned inputs and reports structured results.
- Conformance for generated template variants.

## Phase H2 — Package development diagnostics and reload loop

Make the package development loop visible and repeatable.

Deliverables:

- Manifest diff/diagnostic output for `package check`.
- Dev-loop package restart/reload diagnostics through existing public runtime paths.
- Package logs and status smoke coverage for generated or fixture packages.

## Phase H3 — Composition descriptor v2 diagnostics

Make compositions describe explicit package sets and replacement expectations.

Deliverables:

- Composition descriptor fields for title/description, optional packages, required capabilities, default activation, permission expectations, replacement candidates, and compatibility notes.
- `composition check` diagnostics for missing packages, surfaces, capabilities, entry activation, permission expectations, and replacement candidates.
- `official/composition-lab` output that can summarize launch plans, surface graphs, permission previews, and replacement diagnostics.

## Phase H4 — Forge authoring surfaces

Improve the web shell as an honest public-protocol authoring/inspection surface.

Deliverables:

- Package/surface/capability authoring panels in Forge using only public protocol data.
- Manifest/surface descriptor previews.
- Composition diagnostics display if available through assets/projections/capabilities.
- Proposal review remains approval-gated.

## Phase H5 — Third-party replacement proof

Add a non-official package proving official seeds are replaceable.

Deliverables:

- A third-party playable example package with equivalent launch/Play/Forge/Assist surface shape but different semantics.
- A composition that can choose the official seed or third-party replacement explicitly.
- Conformance proving Home/Forge/Assistant-style discovery and capability invocation do not prefer `official/*`.

## Phase H6 — Documentation and final validation

Update durable guides and status docs, then remove this completed plan document.

Required checks:

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

Also run package checks for representative official and third-party example packages plus a doc-link check.

## Invariants

- Kernel remains content-free.
- Package authorship uses manifests, capabilities, surfaces, hooks, proposals, and protocol calls.
- Official packages remain ordinary packages.
- Forge and generated tools use public protocol paths rather than private runtime internals.
- TavernHeadless remains a reference ledger, not the roadmap driver.
