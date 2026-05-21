# Model Provider Integration Research Ledger

> [English](./README.en.md) · [中文](./README.md)

This directory records the research boundary for Model Provider Integration Alpha. The goal is not to build a relay gateway, billing system, or model proxy; it is to give Yggdrasil ordinary capability packages a grounded path to real model providers.

## Sources researched

- OpenAI Responses / Chat Completions
- Anthropic Messages
- Gemini `generateContent` / `streamGenerateContent`
- OpenAI-compatible providers
- OpenRouter
- DeepSeek
- xAI
- Fireworks
- Reference project: [new-api](https://github.com/Youzini-afk/new-api)
- Reference project: [TavernHeadless](https://github.com/Youzini-afk/TavernHeadless)

## Conclusions for Yggdrasil

- Model providers are ordinary package semantics, not kernel semantics.
- `OpenAI-compatible` is an adapter family, not the whole ontology.
- Anthropic and Gemini need independent dialects; do not force them into an OpenAI delta shape.
- OpenRouter, DeepSeek, xAI, and Fireworks are mostly OpenAI-style but still need provider presets and a quirk layer.
- Usage/cost belongs in package output and outbound audit metadata, not user balances, billing dashboards, or relay ledgers.
- Real egress must go through a host-enforced outbound boundary or equivalent fake/local executor; otherwise secret/network/audit/redaction are voluntary conventions.
- Default conformance uses fake executor/local mocks, not live API keys or external network.
- Manual live calls must be opt-in, use `secret_ref`, network allowlists, redacted audit, and never be CI/release gates.

## Explicit non-goals

- No user balances, recharge, multipliers, channel admin, or admin UI.
- No hosted platform master API key.
- No `kernel.model.*`, `kernel.prompt.*`, `kernel.chat.*`, or `kernel.embedding.*`.
- No provider profile, model catalog, prompt/messages schema in the kernel.
- No implicit network, secret, routing, or UI privilege for official provider packages.

## Files

- [`provider-matrix.yaml`](./provider-matrix.yaml): provider/request/stream/tool/usage/error difference matrix.
- [`stream-compatibility.md`](./stream-compatibility.md): stream event normalization strategy.
- [`error-taxonomy.md`](./error-taxonomy.md): provider error normalization proposal.
- [`new-api-ledger.md`](./new-api-ledger.md): `new-api` lessons to absorb and avoid.
- [`tavern-headless-ledger.md`](./tavern-headless-ledger.md): TavernHeadless provider/profile lessons.
