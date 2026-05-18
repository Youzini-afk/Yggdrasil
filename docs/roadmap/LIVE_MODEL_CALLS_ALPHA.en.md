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

## Phase L4 — First live provider canary

Run one provider through real invoke + stream, preferably DeepSeek / OpenAI-compatible:

- env secret opt-in.
- live invoke.
- live stream.
- auth failure, timeout, rate limit/bad request classification.
- stream cancel/timeout through host boundary.
- Manual `conformance live-model` opt-in; default conformance remains local and stable.

## Phase L5 — OpenAI / Anthropic / Gemini live adapters

Extend three representative non-isomorphic APIs:

- OpenAI Chat/Responses.
- Anthropic Messages named SSE.
- Gemini generateContent / streamGenerateContent.

## Phase L6 — OpenRouter / xAI / Fireworks / DeepSeek quirks

Complete remaining provider families:

- OpenRouter comments + mid-stream error.
- DeepSeek reasoning_content / final usage chunk / keep-alive.
- xAI reasoning timeout / chat vs responses.
- Fireworks responses-style stream fixture.

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
