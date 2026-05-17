# Model Connectivity Kit Alpha

> [English](./MODEL_CONNECTIVITY_KIT_ALPHA.md) · [中文](./MODEL_CONNECTIVITY_KIT_ALPHA.zh-CN.md)

## Purpose

Model Connectivity Kit Alpha introduces Yggdrasil-native model provider connectivity packages without creating a model runtime. It is informed by TavernHeadless provider work, but it is not a Tavern wrapper and it does not perform real inference.

Alpha scope is metadata, validation, redaction-safe profile handling, discovery planning, route planning, compatibility reporting, and parameter normalization.

## Non-goals

- No network requests.
- No API key testing.
- No live model listing.
- No completions, responses, embeddings, or streaming.
- No model/provider terms in kernel types, kernel events, or kernel protocol methods.
- No raw secrets in assets, events, logs, projections, proposals, or UI surfaces.

## Packages

### `official/model-connector-lab`

Provider/profile infrastructure:

- provider family descriptors;
- profile validation;
- secret masking;
- discovery plans;
- compatibility reports.

Capabilities:

- `official/model-connector-lab/describe_families`
- `official/model-connector-lab/validate_profile`
- `official/model-connector-lab/mask_secret`
- `official/model-connector-lab/discovery_plan`
- `official/model-connector-lab/compat_report`

Supported family metadata starts with:

- `openai`
- `openai-compatible`
- `anthropic`
- `google`
- `deepseek`
- `xai`

All outputs must mark live state as `not_verified` or `planned` in Alpha.

### `official/model-routing-lab`

Consumer-slot route planning:

- define and validate consumer slot descriptors;
- resolve static route bindings;
- preview route candidates;
- normalize generation-like parameters;
- explain compatibility/fallbacks.

Capabilities:

- `official/model-routing-lab/define_binding`
- `official/model-routing-lab/resolve_binding`
- `official/model-routing-lab/preview_routes`
- `official/model-routing-lab/params_normalize`
- `official/model-routing-lab/compat_report`

Consumer slots are package-owned labels such as `play.primary` or `analysis.review`; they are not kernel semantics.

## TavernHeadless reference points

TavernHeadless reviewed areas:

- `packages/core/src/llm/provider-registry.ts`
- `packages/core/src/llm/types.ts`
- `packages/core/src/llm/llm-service.ts`
- `apps/api/src/lib/llm-provider-discovery.ts`
- `apps/api/src/routes/llm-profiles.ts`
- `apps/api/src/routes/llm-instances.ts`
- `packages/official-integration-kit/sdk/src/resources/llm-*.ts`

TavernHeadless behavior is reference material. Yggdrasil uses a native package model with no `tavern-*` package names.

## Phase plan

### Phase A — Reference map and fixtures

Add a model connectivity map and compact fixtures under `integrations/tavern-headless/`.

Acceptance:

- reference map targets `official/model-connector-lab` and `official/model-routing-lab`;
- fixtures contain no real secrets;
- docs state no-network/no-inference Alpha scope.

### Phase B — `official/model-connector-lab`

Add manifest, capabilities, in-process deterministic behavior, surfaces, and conformance.

Acceptance:

- profile validation rejects raw secret leakage and malformed base URLs;
- secret masking never returns full values;
- discovery output is a plan, not live results;
- conformance covers supported provider families.

### Phase C — `official/model-routing-lab`

Add manifest, capabilities, in-process deterministic route planning, surfaces, and conformance.

Acceptance:

- route resolution is deterministic;
- fallbacks are explicit;
- params normalization keeps provider-specific options namespaced;
- route plans do not invoke inference.

### Phase D — Guide and status polish

Add a bilingual guide and update README/status/conformance docs.

Acceptance:

- user-facing docs explain how connector and routing labs compose;
- docs clearly defer `model-inference-lab`;
- conformance count is accurate.

### Phase E — Future inference plan and final validation

Document prerequisites for future `model-inference-lab`.

Acceptance:

- future inference requires secret resolution, network permission, request/response audit, streaming/cancel policy, usage accounting, redaction, and provider error taxonomy;
- final validation passes TypeScript, Rust tests, conformance, package checks, and doc-link check.
