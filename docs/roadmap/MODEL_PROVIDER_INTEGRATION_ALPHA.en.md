# Model Provider Integration Alpha

> [English](./MODEL_PROVIDER_INTEGRATION_ALPHA.en.md) · [中文](./MODEL_PROVIDER_INTEGRATION_ALPHA.md)

This is a temporary execution plan. Delete it after completion and fold durable results into the README, `docs/ALPHA_STATUS.md`, `docs/roadmap/NEXT_STEPS.md`, `docs/spec/CONFORMANCE_MATRIX.md`, and guides.

Goal: implement multi-provider model access as ordinary capability packages, covering real API differences across OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, and Fireworks. Default conformance uses fake/local executors, not live network or real keys; manual live calls are opt-in. This is not a relay gateway, billing system, or kernel model/prompt/chat ontology.

## M0 — Research Ledger ✅

- Add `integrations/model-providers/` ledger.
- Fix provider matrix, stream compatibility, and error taxonomy.
- Record `new-api` and TavernHeadless lessons to absorb/avoid.

## M1 — Model Provider Adapter SDK

- Add `sdk/typescript/model-provider-adapter`.
- Define provider profiles, canonical request/response, normalized stream events, usage/cost/error metadata.
- Provide normalization helpers for OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, and Fireworks.

## M2 — `official/model-provider-lab` no-network normalization

- Add an ordinary official package with `list_supported_families`, `validate_profile`, `normalize_request`, `explain_error`, and `echo`.
- No network, no real inference.

## M3 — Host Outbound Executor Boundary

- Add a content-free outbound executor abstraction: request/response/fake executor/local mock support.
- Keep default deny/fake; real egress requires explicit opt-in.
- Enforce network allowlist, secret_ref, redacted audit, timeout/cancel.

## M4 — OpenAI / Anthropic / Gemini invoke adapters

- Implement fake/local invoke paths for three representative non-compatible families inside `model-provider-lab`.
- Support manual live call paths, but keep them out of default conformance.

## M5 — OpenAI-compatible / OpenRouter / DeepSeek / xAI / Fireworks presets

- Add provider presets, base URL/header quirks, usage/error mapping.
- OpenAI-compatible is an adapter family, not the only protocol.

## M6 — Streaming normalization

- Normalize delta SSE, semantic SSE, and typed chunk streams into provider package normalized stream events, then wrap them as `StreamFrameEnvelope`.
- Cover terminal/error/usage/cancel/timeout.

## M7 — Examples, conformance, durable docs, cleanup

- Add provider profile examples, manual live smoke docs, and conformance.
- Add `docs/guides/MODEL_PROVIDER_INTEGRATION.md` / `.en.md`.
- Delete this temporary plan.

## Non-goals

- No user balances, billing, channel admin, or admin UI.
- No hosted platform relay key.
- No `kernel.model.*`, `kernel.prompt.*`, `kernel.chat.*`, or `kernel.embedding.*`.
- No implicit secret/network/routing/UI privilege for official provider packages.
