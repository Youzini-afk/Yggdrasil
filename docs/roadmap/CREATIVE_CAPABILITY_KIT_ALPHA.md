# Creative Capability Kit Alpha

> [English](./CREATIVE_CAPABILITY_KIT_ALPHA.md) · [中文](./CREATIVE_CAPABILITY_KIT_ALPHA.zh-CN.md)

## Purpose

Creative Capability Kit Alpha turns mature headless RP/tooling ideas into Yggdrasil-native, general-purpose official capability packages. TavernHeadless is treated as a reference corpus and test oracle, not as Yggdrasil ontology.

The outcome must be useful for AI games, RP, interactive fiction, simulations, worldbuilding, agents, and external engines. It must not create `tavern-*` packages or move prompt/persona/knowledge concepts into the kernel.

## Non-negotiable boundaries

- Kernel impact is zero unless a bug is found in generic package/capability/surface mechanics.
- No kernel type, event, or protocol method may mention `persona`, `character`, `prompt`, `context`, `worldbook`, `lorebook`, `chat`, `message`, `turn`, `Tavern`, or model calls.
- Official packages are ordinary packages: same manifest, same capability routing, same permissions, same surface descriptors.
- Import/export compatibility is secondary. The core package interfaces must be useful without SillyTavern or TavernHeadless.
- All generated plans must include provenance, diagnostics, and approval-gated proposal shapes when mutation is implied.

## Reference source

TavernHeadless capabilities reviewed:

- character card parsing/export and unknown-field preservation;
- worldbook/lorebook normalization, trigger logic, recursion, and outlet placement;
- preset parsing, prompt-order semantics, compat assembly, native prompt graph compilation;
- regex profile parsing and deterministic transform traces;
- prompt runtime traces, source selection, budget pruning, and template rendering;
- public SDK/OpenAPI boundaries and CI/version-check discipline.

These ideas are adapted into generic packages:

- `official/persona-lab`
- `official/knowledge-lab`
- `official/context-lab`
- `official/text-transform-lab`

## Normalized capability package targets

### `official/persona-lab`

Purpose: import, normalize, describe, validate, and render persona-like structured profiles without assuming chat characters.

Capabilities:

- `official/persona-lab/import_profile`
- `official/persona-lab/normalize_profile`
- `official/persona-lab/describe_profile`
- `official/persona-lab/render_fragment`
- `official/persona-lab/compat_report`

Output kind examples: `persona_profile`, `persona_fragment`, `persona_compat_report`.

### `official/knowledge-lab`

Purpose: manage structured knowledge collections and deterministic activation/matching plans without making lorebook/worldbook semantics canonical.

Capabilities:

- `official/knowledge-lab/import_collection`
- `official/knowledge-lab/normalize_entries`
- `official/knowledge-lab/match_entries`
- `official/knowledge-lab/injection_plan`
- `official/knowledge-lab/compat_report`

Output kind examples: `knowledge_collection`, `knowledge_match_result`, `knowledge_injection_plan`.

### `official/context-lab`

Purpose: assemble bounded context blocks from explicit sources, budgets, and policies for any downstream consumer.

Capabilities:

- `official/context-lab/assemble_preview`
- `official/context-lab/inspect_layers`
- `official/context-lab/budget_plan`
- `official/context-lab/render_template`
- `official/context-lab/explain_assembly`

Output kind examples: `context_preview`, `context_layer_inspection`, `context_budget_plan`.

### `official/text-transform-lab`

Purpose: deterministic text transforms, templates, regex-like rules, macro imports, pipeline explanations, and compatibility diagnostics.

Capabilities:

- `official/text-transform-lab/import_rules`
- `official/text-transform-lab/validate_rules`
- `official/text-transform-lab/apply_preview`
- `official/text-transform-lab/explain_pipeline`
- `official/text-transform-lab/compat_report`

Output kind examples: `text_transform_profile`, `text_transform_preview`, `text_transform_pipeline`.

## Upstream tracking

Create `integrations/tavern-headless/` as a reference ledger, not a runtime dependency:

- `upstream.lock.toml`: reviewed path/ref/version/date/toolchain.
- `capability-map.yaml`: maps TavernHeadless subsystems to Yggdrasil-native packages with `adapted|deferred|adapter_only|rejected` status.
- `README.md`: explains that TavernHeadless is a reference source.
- `fixtures/`: compact examples for character cards, knowledge books, presets/context, and text transform rules.

Future update checks should compare the reviewed TavernHeadless commit and changed subsystem paths, then decide whether to adopt, adapt, defer, or reject changes.

## Phase plan

### Phase A — Reference tracking and fixtures

Add the integration ledger and compact fixtures. Add docs that explain the abstraction from TavernHeadless into Yggdrasil-native packages.

Acceptance:

- no product package is named `tavern-*`;
- fixtures are small and repository-local;
- reference map points to Yggdrasil package names;
- docs state that compatibility input is not canonical ontology.

### Phase B — `official/persona-lab`

Add ordinary manifest, capabilities, surfaces, in-process behavior, conformance, Forge visibility, and docs.

Acceptance:

- imports a profile/card-like payload into normalized profile output;
- preserves unknown fields in diagnostics;
- render fragment includes provenance;
- no direct asset mutation.

### Phase C — `official/knowledge-lab`

Add ordinary package for knowledge collections, entry normalization, matching, and injection planning.

Acceptance:

- deterministic keyword matching with trace;
- injection plan output stays a plan, not implicit context mutation;
- compatibility report can describe worldbook-like inputs without canonizing them.

### Phase D — `official/context-lab`

Add ordinary package for context previews, layer inspection, template rendering, and budget planning.

Acceptance:

- output uses generic context blocks, not chat messages;
- included and omitted sources have reasons;
- budget accounting is visible;
- no model calls.

### Phase E — `official/text-transform-lab` and guide polish

Add ordinary package for transform rule import/validation/preview and pipeline explanation. Update bilingual guide, README, status, conformance matrix, and UI surface wording if needed.

Acceptance:

- deterministic transform preview includes trace;
- unsafe/unsupported rules produce diagnostics;
- conformance grows for all four packages;
- validation passes TypeScript, Rust tests, conformance, package checks, and doc links.

## Validation gate

Every phase must pass its scoped checks before commit/push. Final gate:

```bash
tsc -p clients/web/tsconfig.json --noEmit
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

Package checks must run for every new official package.
