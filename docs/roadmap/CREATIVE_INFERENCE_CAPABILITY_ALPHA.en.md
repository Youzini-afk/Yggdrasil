# Creative Inference Capability Alpha

> [English](./CREATIVE_INFERENCE_CAPABILITY_ALPHA.en.md) · [中文](./CREATIVE_INFERENCE_CAPABILITY_ALPHA.md)

## Position

Yggdrasil's near-term delivery path is **API-first**, because most players and creators get model capability through cloud APIs today. But Yggdrasil must not become **API-shaped**: HTTP, bearer tokens, JSON schemas, and OpenAI/Anthropic/Gemini request shapes are cloud adapter package details, not platform worldview.

Architecture principles:

- Cloud APIs are the first adapter targets, not platform primitives.
- `official/model-provider-lab` is an ordinary cloud API adapter lab, not the Yggdrasil model abstraction.
- Do not keep proving the platform by adding more providers; the current eight cloud providers already prove realistic API integration.
- Next, prove how inference participates in Yggdrasil's session / branch / proposal / inspection / fork creative runtime.
- Preserve seams for local/self-hosted/non-HTTP providers, but do not build a full local model platform in this phase.

## Non-goals

- No LiteLLM / OneAPI style relay gateway.
- No user balances, billing, channel admin, or provider marketplace.
- No model downloader, weight cache, GPU scheduler, or full llama.cpp/vLLM/Ollama integration.
- No `kernel.model.*`, `kernel.prompt.*`, `kernel.chat.*`, or `kernel.embedding.*`.
- No unified chat schema, messages/system/user/assistant shape, or OpenAI-compatible request as a public platform protocol.

## Phase C0 — ADR and plan (complete)

Deliver:

- This temporary plan.
- `NEXT_STEPS` / `ALPHA_STATUS` point to Creative Inference Capability Alpha.
- Explicit “API-first but not API-shaped” and cloud adapter downgrade principles.

Acceptance:

- Documentation links pass.
- No new kernel model/prompt/chat terms as protocol or code methods.

## Phase C1 — Transport-neutral inference capability contract (complete)

Goal: define inference as an ordinary package/capability-layer contract, not a kernel feature.

Delivered:

- `sdk/typescript/inference-capability`: transport-neutral envelope and stream/error/capability manifest helpers. Includes `InferenceRequest`/`InferenceResponse`/`InferenceStreamFrame`/`InferenceError` types, `createInferenceRequest`/`validateInferenceRequest`/`classifyInferenceError`/`InferenceStreamLifecycle`/`createProviderCapabilityManifest`/`validateProviderCapabilityManifest` helper functions, 69 pure-TS self-tests passing.
- `docs/guides/INFERENCE_CAPABILITY_AUTHORING.md` and `.en.md`: package authoring guide.
- The contract does not require URL/header/status-code/OpenAI messages fields.
- Error taxonomy covers cloud (authentication/rate_limit/billing/provider_overloaded/…) and local/resource (local_process_failed/local_resource_exhausted/local_model_not_loaded/…) errors.
- Provider capability manifest supports modality/transport/secret/network/local runtime hints.

The contract expresses:

- operation id / operation kind;
- input artifacts or opaque input payload refs;
- streaming / non-streaming;
- deadline / cancellation;
- resource hints;
- secret refs;
- transport kind hint (`http`, `local_process`, `in_memory`, `ipc`, `websocket`, `remote`, `custom`);
- canonical stream frames;
- transport-neutral error taxonomy.

## Phase C2 — Non-HTTP fake local provider proof

Goal: prove the inference pipeline does not depend on HTTP, bearer tokens, or JSON provider schemas.

Candidate deliverables:

- `packages/official/inference-local-lab`: deterministic non-HTTP fake inference provider.
- Capabilities: `describe_capabilities`, `invoke`, `stream`, `explain_error`.
- Conformance: deterministic stream frames without URL, Authorization, HTTP status, or provider schema.

This is not a local model platform; it is a seam proof that prevents the abstraction from hardening into an HTTP proxy.

## Phase C3 — Cloud adapter package reposition

Goal: downgrade existing `official/model-provider-lab` into a cloud adapter package, not a platform abstraction.

Candidate deliverables:

- Documentation and manifest descriptions call it a cloud API adapter lab.
- `MODEL_PROVIDER_INTEGRATION` adds negative claims: not the Ygg model abstraction, not an API gateway, no kernel privilege.
- `normalize_request` is described only as a package-local adapter helper, not a canonical platform schema.
- Conformance wording changes from “model provider abstraction” to “cloud adapter coverage”.

## Phase C4 — Ygg-native inference proposal vertical slice

Goal: prove inference is not “prompt -> text response”, but participation in the Yggdrasil creative runtime.

Candidate deliverables:

- `packages/official/inference-playtest-lab` or an extension of `inference-local-lab`.
- Flow: session state → inference capability → proposal → inspect → approve/reject → apply → branch/fork → replay/audit.
- Output must be an approval-gated proposal or package-owned events, not a chat message.
- Provider can be fake local or cloud adapter; the vertical slice must not require public internet.

## Phase C5 — Durable docs cleanup

Goal: delete the temporary plan and fold outcomes into durable guides, status, matrix, and README.

Deliver:

- Delete this temporary plan.
- Update `docs/guides/INFERENCE_CAPABILITY_AUTHORING.md`, `MODEL_PROVIDER_INTEGRATION.md`, `ALPHA_STATUS`, `NEXT_STEPS`, and `CONFORMANCE_MATRIX`.
- Full validation and push.

## Risk controls

- Kernel remains content-free.
- Official inference/cloud packages have no private outbound or routing privilege.
- Cloud adapters remain usable but do not define the platform abstraction.
- Non-HTTP proof must stay small and must not become a premature local model platform.
