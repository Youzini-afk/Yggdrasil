# Model Provider Integration

> [English](./MODEL_PROVIDER_INTEGRATION.en.md) · [中文](./MODEL_PROVIDER_INTEGRATION.md)

This guide documents Yggdrasil's model provider integration path. It is not a relay gateway, billing system, provider admin backend, or kernel model abstraction. Model integration must be delivered as ordinary capability packages using the same manifest, permission, `secret_ref`, outbound audit, stream/cancel, and conformance boundaries as every other package.

## Current delivery

Model Provider Integration Alpha is complete:

- `integrations/model-providers/` stores the provider research ledger, provider matrix, stream compatibility notes, and error taxonomy.
- `sdk/typescript/model-provider-adapter` provides a pure TypeScript adapter for provider profiles, request normalization, error classification, and stream event parsing. It does not go online, do billing, or access private runtime APIs.
- `official/model-provider-lab` is an ordinary official capability package covering eight provider families: OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, and Fireworks.
- `official/model-provider-lab` exposes `list_supported_families`, `validate_profile`, `normalize_request`, `invoke`, `normalize_stream`, `explain_error`, and `echo`.
- `invoke` is currently a fake/local provider adapter path: it returns provider-shaped responses and auditable `outbound_request_shape`, while keeping `network_performed=false`, `inference_performed=false`, and `executor_kind=fake_local`.
- The host has a content-free `OutboundExecutor` boundary. It defaults to deny-all and has fake executor conformance. This proves request shapes can flow through host policy/audit boundaries, but it does not claim OS-level interception of arbitrary subprocess networking.
- `normalize_stream` maps delta SSE, semantic SSE, and typed chunk streams into `StreamFrameEnvelope`-style frames: `start`, `chunk`, `progress`, `end`, `error`, `cancelled`, and `timeout`.

## Provider families

| Family | Request dialects | Stream shape | Default host |
| --- | --- | --- | --- |
| `openai` | `openai_chat`, `openai_responses` | `delta_sse`, `semantic_sse` | `api.openai.com` |
| `anthropic` | `anthropic_messages` | `semantic_sse` | `api.anthropic.com` |
| `gemini` | `gemini_generate_content` | `typed_chunk_stream` | `generativelanguage.googleapis.com` |
| `openai_compatible` | `openai_chat` | `delta_sse` | explicit `baseUrl` required |
| `openrouter` | `openai_chat`, `stateless_responses` | `delta_sse`, `semantic_sse` | `openrouter.ai` |
| `deepseek` | `openai_chat` | `delta_sse` | `api.deepseek.com` |
| `xai` | `openai_chat`, `openai_responses` | `delta_sse`, `semantic_sse` | `api.x.ai` |
| `fireworks` | `openai_chat`, `fireworks_responses` | `delta_sse`, `semantic_sse` | `api.fireworks.ai` |

OpenAI-compatible is an adapter family, not Yggdrasil's only model worldview. Anthropic, Gemini, and Responses-style streams have different structures and must be adapted explicitly.

## Profile shape

Example profiles live under `examples/model-provider-profiles/`. All examples use `secret_ref`; none contain real keys.

```json
{
  "family": "anthropic",
  "model": "claude-3-5-sonnet-20241022",
  "credential": "secret_ref:env:ANTHROPIC_API_KEY",
  "baseUrl": "https://api.anthropic.com",
  "headers": {
    "anthropic-version": "2023-06-01"
  }
}
```

Rules:

- `credential` must be a `secret_ref:*`, `secretRef:*`, `secret-ref:*`, or `host:*` reference.
- Raw credentials, raw-looking headers, `api_key`, and `secret` fields are rejected.
- `invoke` requires an HTTPS `baseUrl`; `openai_compatible` has no default host and must provide an explicit HTTPS `baseUrl`.
- Profiles may carry provider-specific `headers` and `extra`, but these remain package semantics and do not become kernel ontology.

## Common capabilities

### `list_supported_families`

Returns the eight families with default base URL, request dialect, stream family, credential header, and notes. It does not go online.

### `validate_profile`

Checks family, model, credential, base URL, and headers, returning:

- `valid`
- `diagnostics`
- `network_required`
- `secret_refs`
- `network_performed=false`
- `inference_performed=false`

### `normalize_request`

Turns common input into provider-specific request shapes, such as:

- OpenAI Chat: `/v1/chat/completions`
- OpenAI Responses: `/v1/responses`
- Anthropic: `/v1/messages`
- Gemini: `/v1beta/models/{model}:generateContent`
- OpenRouter: `/chat/completions` or `/responses`
- DeepSeek: `/chat/completions`
- xAI: `/v1/chat/completions` or `/v1/responses`
- Fireworks: `/chat/completions` or `/responses`

### `invoke`

Currently does not perform real network calls. It returns provider-shaped fake/local responses plus an `outbound_request_shape`, proving adapter, secret_ref, base URL, method, path, and redaction shape.

Outputs must keep:

```json
{
  "network_performed": false,
  "inference_performed": false,
  "executor_kind": "fake_local",
  "live_call_supported": false
}
```

Real live calls are future ordinary package/host-policy work and must not be granted through private official-package paths.

### `normalize_stream`

Maps provider events/chunks into `StreamFrameEnvelope`-style frames. Provider-specific fields stay in `payload` or `metadata`:

- OpenAI Chat / OpenAI-compatible / OpenRouter / DeepSeek / xAI / Fireworks: delta SSE.
- OpenAI Responses / Anthropic: semantic SSE.
- Gemini: typed chunk stream; `GenerateContentResponse`-style snapshots are adapted into chunk/end/progress frames.

Structural events such as Anthropic `message_start`, `content_block_start`, OpenRouter comments, and Gemini snapshots with no new text become `progress`, not text chunks. Usage, finish reasons, safety ratings, performance metrics, and generation IDs go into `metadata` or end payload, not kernel semantics.

### `explain_error`

Normalizes provider-specific code/status values into package-level error categories, such as:

- `authentication`
- `permission`
- `rate_limit`
- `billing`
- `not_found`
- `bad_request`
- `tool_schema`
- `timeout`
- `overloaded`
- `stream_error`
- `upstream_malformed`
- `unknown`

This is package-level normalization, not `kernel.model.error`.

## Manual live call boundary

Alpha does not execute real provider requests by default. Future live paths must satisfy:

1. the provider package declares minimal network permissions;
2. caller or host policy explicitly allows them;
3. secrets are resolved through host resolver, with raw secrets never entering events/logs/audits;
4. requests go through the host outbound boundary;
5. audits record only host, method, purpose, secret_refs, usage/cost/error metadata, and redaction state;
6. streams map to the content-free frame lifecycle;
7. cancel/timeout are not swallowed by provider adapters;
8. third-party provider packages can replace official packages without official priority.

## Non-goals

- User balances, top-ups, billing admin, multipliers, or channel management systems.
- Hosted platform relay keys.
- `kernel.model.*`, `kernel.prompt.*`, `kernel.chat.*`, or `kernel.embedding.*`.
- Treating OpenAI-compatible as the only model protocol.
- Letting official packages bypass manifest, permission, secret, network, or audit boundaries.

## Validation

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- package check packages/official/model-provider-lab/manifest.yaml
tsc -p clients/web/tsconfig.json --noEmit
```

Current conformance includes `official.model_provider_lab`, `official.model_provider_lab_invoke_core`, `official.model_provider_lab_normalize_stream`, and outbound fake executor hostile cases.
