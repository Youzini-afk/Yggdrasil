# Live Model Calls Alpha

> [English](./LIVE_MODEL_CALLS_ALPHA.en.md) · [中文](./LIVE_MODEL_CALLS_ALPHA.md)

## Goal

Advance the Model Provider Integration Alpha fake/local provider path into a real live-call path:

```text
secret_ref → host secret resolver → public outbound boundary → live HTTPS executor → provider adapter → normalized response/stream → redacted audit
```

This is not a relay gateway, billing system, channel admin, or kernel model ontology. Live model calls remain ordinary capability-package behavior, with equal runtime rights for official and third-party packages.

## Invariants

- Do not add `kernel.model.*`, `kernel.prompt.*`, `kernel.chat.*`, or `kernel.embedding.*`.
- Provider packages must not read env, `.env`, credential stores, or raw keys directly.
- Provider packages must not use `reqwest`/`fetch`/`curl`/provider SDKs directly to bypass the host outbound boundary.
- Official provider packages must not use private runtime network APIs; third-party packages must be able to use the same public boundary.
- Default CI/conformance must not depend on the internet or real keys; live tests must be opt-in.
- Raw secrets must not enter events, audit, logs, errors, stream frames, fixtures, or doc examples.
- The live executor is disabled by default; host profiles must explicitly allow env secrets and outbound host/methods.
- Real HTTPS must go through the host-controlled executor and enforce timeout, redirect policy, redaction, and audit.

## Phase L0 — contract freeze (this file)

Deliverables:

- This temporary plan.
- Live-call contract: secret resolver, public outbound, redacted audit, live-test opt-in, provider no-direct-network.
- Roadmap/status point to Live Model Calls Alpha.

Validation:

- Doc link check.
- `kernel.model/prompt/chat/embedding` only appear in non-goals/prohibitions.

## Phase L1 — Host EnvSecretResolver ✅

Implemented host-owned env secret resolver:

- Supports `secret_ref:env:NAME` / `secretRef:env:NAME` / `secret-ref:env:NAME` / `host:env:NAME`.
- Deny-all default; host config must explicitly allow env names (allowlist-only).
- Missing, denied, and malformed references return typed errors.
- Raw values exist only transiently inside the host and are never serialized; audit/errors include only references/env names, never values.
- `Runtime::resolve_secret_ref` host-internal method for host use during capability invocation.
- `extract_env_name` helper recognizes only `env` vault; `host:<key>` without `env:` prefix not treated as env.
- Conformance covers allowed/missing/denied/no-leak (3 new cases: `secret.env_resolver_allowed`, `secret.env_resolver_denied`, `secret.env_resolver_missing_no_leak`).

## Phase L2 — LiveHttpOutboundExecutor ✅

Implemented host live HTTP executor:

- `reqwest + rustls`, no native-tls.
- Disabled by default; `RuntimeConfig` must opt in via `OutboundExecutorConfig::LiveHttp(config)`.
- HTTPS-only; non-HTTPS URLs are rejected (fail-closed).
- Redirects rejected by default (`allow_redirects: false`); L2 does not implement redirect following, and `allow_redirects=true` fails closed until redirect target policy re-check lands.
- Configurable connect/request timeouts; streaming idle watchdog deferred until stream phase.
- Headers/body recorded only as shape/audit metadata, never raw auth/body.
  - Only sends `content-type: application/json` and `x-ygg-outbound` placeholder headers.
  - L2 does not inject secrets (L3 handles injection through the host boundary).
  - Response `headers_shape` records only content-type and request-id safe header values; all other header values are replaced with `[redacted]`.
  - Response `body_shape`: JSON secret fields replaced with `[redacted]`; non-JSON records kind/bytes_captured only.
- Denied/policy-mismatch requests never call the executor.
- Errors normalized to `status="error"` or `"timeout"`, with no raw body/secret content.
- `allow_insecure_loopback_for_tests` defaults to false; permits only 127.0.0.1/localhost http:// URLs for conformance testing.
- `LiveHttpOutboundExecutorConfig` provides `timeout_ms`, `connect_timeout_ms`, `allow_redirects`, `max_response_preview_bytes`, `allow_insecure_loopback_for_tests`.
- Conformance adds 3 new cases: `outbound.live_http_default_disabled`, `outbound.live_http_rejects_insecure_url`, `outbound.live_http_redacted_shape`. No public internet dependency.

## Phase L3 — Public outbound/secret boundary ✅

Expose content-free host boundary to ordinary capability packages:

- `kernel.outbound.execute` public protocol method: allows ordinary packages to make outbound requests through the host outbound executor. Params accept capability_id, destination_host, method, path, secret_refs, metadata, body_shape. package_id is enforced from the ProtocolContext principal — callers cannot spoof a different package_id in params (host_dev/host_admin principals may specify package_id in params for testing). Dispatch calls `execute_outbound_with_policy`; response undergoes additional defense-in-depth raw-secret sweep. Does not add `kernel.secret.resolve` (raw secrets are never returned to packages). L3 does not inject secret headers (real injection deferred to L4/L5).
- Official and third-party provider packages use the same path.
- Docs clarify arbitrary subprocess networking is still not OS-level intercepted; uncontrolled subprocess providers are not default live providers.
- Conformance adds 4 new cases: `outbound.execute_package_allowed`, `outbound.execute_spoofed_package_id_rejected`, `outbound.execute_no_permission_denied`, `outbound.execute_no_raw_secret_in_response`. No public internet dependency.

## Phase L4 — First live provider canary invoke+stream ✅

Implemented the minimum verifiable path for first live provider canary invoke+stream:

- **Host-side secret header injection**: `kernel.outbound.execute` gains a `secret_headers` param with format `{ "Authorization": {"secret_ref":"secret_ref:env:DEEPSEEK_API_KEY", "scheme":"bearer"}}`. Host resolves the secret_ref via `EnvSecretResolver` and constructs the header value (e.g. `Bearer <key>`), injecting it into `LiveHttpOutboundExecutor` HTTP request headers. Raw secrets are never returned to packages, audit, errors, or responses.
- **`OutboundExecutorRequest` extension**: New fields `secret_headers: Vec<SecretHeaderSpec>` (parsed spec) and `resolved_secret_headers: Vec<ResolvedSecretHeader>` (host-resolved values, wrapped in `RedactedHeaderValue` newtype, Debug/Serialize do not leak).
- **`LiveHttpOutboundExecutor::build_headers` injection**: L4 reads `resolved_secret_headers` and injects Authorization and other secret headers; raw values exist only in the HTTP request, never in audit/Debug/response shapes.
- **Protocol dispatch L4 integration**: `parse_secret_headers` parses `secret_headers` params; `resolve_secret_ref` resolves each secret_ref; resolved headers flow into `OutboundExecutorRequest`; secret_refs from secret_headers are merged into `all_secret_refs` for policy/audit.
- **Canary provider profile shape**: `model-provider-lab/normalize_request` validates DeepSeek profile maps to the correct endpoint (api.deepseek.com), request_dialect (openai_chat), stream_family (delta_sse).
- **SSE stream canary**: `model-provider-lab/normalize_stream` validates DeepSeek delta_sse normalizes to start→chunk→end frames, terminal_frame_consistent=true, network_performed=false, no raw secrets.
- **Local loopback HTTP server conformance**: Starts a local HTTP server (loopback only, no public internet), verifies Authorization header actually arrives at server, but raw secret does not appear in protocol response/audit/log. Uses `allow_insecure_loopback_for_tests=true`.
- **Opt-in live conformance**: Only attempts real `kernel.outbound.execute` when `YGG_LIVE_MODEL_TESTS=1` AND `DEEPSEEK_API_KEY` is set. Default conformance skips (no public internet dependency).
- Conformance adds 5 new cases: `outbound.secret_headers_parsed`, `outbound.live_loopback_secret_injection`, `stream.sse_normalize_deepseek_canary`, `outbound.live_deepseek_opt_in`, `canary.deepseek_profile_shape`. No public internet dependency.

**L4 does not cover** (deferred to L5):
- Real provider streaming through outbound boundary (current stream canary proves host boundary path via normalize_stream; real HTTP SSE streaming deferred to L5).
- Real provider auth failure/timeout/rate limit classification.
- Multi-provider live adapters (OpenAI/Anthropic/Gemini deferred to L5).

## Phase L5 — OpenAI / Anthropic / Gemini live adapters ✅

Extended three representative non-isomorphic APIs through the public `kernel.outbound.execute` boundary:

- **OpenAI Chat Completions** (`/v1/chat/completions`): Authorization bearer secret_ref injection, messages body shape. Loopback conformance verifies Bearer header arrives at server, POST method, correct path, no raw secret in response/audit.
- **OpenAI Responses** (`/v1/responses`): Same Authorization bearer scheme, different endpoint and body shape (uses `input` instead of `messages`). Loopback conformance verifies distinct endpoint routing.
- **Anthropic Messages** (`/v1/messages`): `x-api-key` secret header injection (raw scheme, no Bearer prefix) + `anthropic-version` safe static header (allowlisted, non-secret). Loopback conformance verifies both headers arrive at server, POST method, content blocks body shape, no raw secret leaks.
- **Gemini generateContent** (`/v1beta/models/{model}:generateContent`): `x-goog-api-key` secret header injection (raw scheme). Loopback conformance verifies header arrives, POST method, colon-style path, contents/parts body shape, no raw secret leaks.
- **Missing secret fails closed**: When a `secret_headers` reference cannot be resolved, `kernel.outbound.execute` returns an error, no outbound request is made, and no raw secret appears in the error.
- **Provider normalize_request alignment**: `model-provider-lab/normalize_request` outputs for OpenAI (chat+responses), Anthropic (messages), and Gemini (generateContent) correctly map to the expected `kernel.outbound.execute` params (host, method, path, header names). Credential placeholders are safe refs, not raw values. Provider packages use the same public boundary; no private runtime calls.
- **No raw secret leak across all providers**: OpenAI/Anthropic/Gemini request shapes through FakeOutboundExecutor produce responses and audit events with zero raw secret content.
- **Safe `static_headers` support (L5)**: `kernel.outbound.execute` gains a `static_headers` param for safe non-secret header injection. Only a tiny `STATIC_HEADER_ALLOWLIST` is accepted (anthropic-version, content-type, accept). Known secret-bearing header names (Authorization, x-api-key, x-goog-api-key, Cookie, etc.) are explicitly blocked — these must use `secret_headers` with `secret_ref` instead; host-owned headers such as x-ygg-outbound, user-agent, and accept-encoding cannot be overridden by packages. Static header values are checked for raw-secret-like patterns. This prevents `static_headers` from becoming a secret bypass or host header override path.
- **`OutboundExecutorRequest` extension**: New `static_headers: Vec<StaticHeader>` field carries validated safe headers.
- **`LiveHttpOutboundExecutor::build_headers` L5**: Injects `static_headers` into the HTTP request alongside secret headers and default headers.
- Conformance adds 9 new cases: `outbound.openai_chat_loopback`, `outbound.openai_responses_loopback`, `outbound.anthropic_messages_loopback`, `outbound.gemini_generate_content_loopback`, `outbound.missing_secret_fails_closed`, `outbound.provider_normalize_request_alignment`, `outbound.no_raw_secret_leak_all_providers`, `outbound.static_headers_safe_allowlist`, `outbound.static_headers_block_secrets`. No public internet dependency.

## Phase L6 — OpenRouter / xAI / Fireworks / DeepSeek quirks ✅

Complete remaining provider families:

- **OpenRouter**: Authorization bearer secret_ref + safe static headers (`http-referer`, `x-title` added to `STATIC_HEADER_ALLOWLIST`); loopback verifies Authorization + HTTP-Referer + X-Title all arrive at server, POST `/api/v1/chat/completions`, raw secret not in response/audit.
- **xAI**: Authorization bearer secret_ref, loopback verifies `/v1/chat/completions` path, Bearer header arrives; reasoning/usage fields sanitized.
- **Fireworks**: Authorization bearer secret_ref, loopback verifies `/inference/v1/chat/completions` path, Bearer header arrives; perf/usage metadata sanitized.
- **DeepSeek quirks**: `normalize_stream` enhanced with `reasoning_content`, final usage chunk (`prompt_cache_hit_tokens`/`prompt_cache_miss_tokens`), SSE keep-alive comments (`": keep-alive"` → progress heartbeat), mid-stream error events (`{"error": {...}}` → error frame).
- **Sanitized fixtures**: 4+ `.json` fixtures under `integrations/model-providers/fixtures/` (`deepseek_reasoning_stream.json`, `openrouter_midstream_error.json`, `xai_reasoning_stream.json`, `fireworks_perf_stream.json`, `openrouter_outbound_shape.json`), all containing no real keys or provider-looking raw keys, with bilingual sanitized documentation.
- **`STATIC_HEADER_ALLOWLIST` extended**: Added `http-referer` and `x-title` (case-insensitive), supporting OpenRouter attribution and request labeling headers. `is_secret_header_name` confirms these are not secret-bearing; Authorization/x-api-key remain blocked.
- **`normalize_stream` enhanced**: New `normalize_deepseek_event`, `normalize_xai_event`, `normalize_fireworks_event` provider-specific normalizers covering reasoning_content, cache usage, perf usage, latency metadata, reasoning usage. SSE keep-alive comments and mid-stream error events handled uniformly at the `normalize_provider_events` entry point (universal across all provider families).
- Conformance adds 7 new cases: `outbound.openrouter_loopback_headers`, `outbound.xai_loopback`, `outbound.fireworks_loopback`, `stream.deepseek_reasoning_stream`, `stream.openrouter_midstream_error`, `outbound.provider_quirk_fixtures_no_secrets`, `outbound.static_headers_openrouter_safe`. No public internet dependency.

## Phase L7 — durable docs + cleanup

Consolidate into long-lived docs:

- `docs/guides/LIVE_MODEL_CALLS.md`.
- live setup examples (only `secret_ref`, no raw keys).
- Update README / ALPHA_STATUS / NEXT_STEPS / CONFORMANCE_MATRIX.
- Delete this temporary plan.

## Acceptance criteria

Alpha is complete when it proves:

1. At least one provider can live invoke.
2. At least one provider can live stream.
3. All live requests go through the host outbound executor.
4. Unauthorized host/method is denied.
5. Provider packages cannot read env secrets directly.
6. EnvSecretResolver/HostSecretResolver can inject keys without leaking raw values.
7. Audit events cover the call lifecycle and are redacted.
8. Live conformance is opt-in; default conformance remains offline.
9. Official providers have no private outbound privilege and third-party providers can use the same path.
