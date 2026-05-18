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

## Phase L1 — Host EnvSecretResolver

Implement host-owned env secret resolver:

- Support `secret_ref:env:NAME` / `secretRef:env:NAME` / `secret-ref:env:NAME` / `host:env:NAME`.
- Deny-all by default; host config must explicitly allow env names.
- Missing, denied, and malformed references return typed errors.
- Raw values exist only transiently inside the host and are never serialized; audit/errors include only references/env names, never values.
- Conformance covers allowed/missing/denied/no-leak.

## Phase L2 — LiveHttpOutboundExecutor

Implement host live HTTP executor:

- `reqwest + rustls`.
- Disabled by default; `RuntimeConfig` must opt in.
- HTTPS-only.
- Redirects rejected by default, or limited to same-host/explicit allowlist.
- Connect/request timeout; streaming idle watchdog deferred until stream phase.
- Headers/body recorded only as shape/audit metadata, never raw auth/body.
- Denied/policy-mismatch requests never call the executor.
- Default conformance uses local loopback fixtures or fake, never public internet.

## Phase L3 — Public outbound/secret boundary

Expose content-free host boundary to ordinary capability packages:

- `kernel.secret.resolve` or equivalent host protocol method (returns redacted/host-usable handles, not raw keys to packages).
- `kernel.outbound.execute` / `kernel.outbound.stream` or equivalent capability-facing methods.
- Official and third-party provider packages use the same path.
- Docs clarify arbitrary subprocess networking is still not OS-level intercepted; uncontrolled subprocess providers are not default live providers.

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
