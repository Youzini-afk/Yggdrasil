# Model Inference Prerequisites

> [English](./MODEL_INFERENCE_PREREQUISITES.md) · [中文](./MODEL_INFERENCE_PREREQUISITES.zh-CN.md)

Model Connectivity Kit Alpha deliberately stops before real model execution. A future `official/model-inference-lab` or equivalent package family must not start until these prerequisites are specified and covered by conformance.

## Required platform contracts

1. **Secret resolution**
    - Profile assets may reference secrets, but raw secrets must not appear in events, projections, logs, UI, or assistant proposals.
    - Hosts need a public secret-reference capability or policy surface.
    - **Phase S1 progress**: `SecretRef` type with `secret_ref:`, `secretRef:`, `secret-ref:`, `host:` patterns. `HostSecretResolver` trait and `DenyAllSecretResolver` placeholder. Raw-secret blocking in proposals and asset metadata. Official packages have no bypass. Permission grants survive rehydrate. Production vault integration remains host-level.

2. **Network permission**
    - Packages need explicit network permissions by destination, method, and purpose.
    - No package should infer network permission from being official.
    - **Phase S2 progress**: Manifest `permissions.network` supports structured `declarations` (host, methods, purpose) and flat `hosts` for backward compat. Runtime `check_network_policy` and `check_and_audit_outbound` enforce allowlists. Official packages have no bypass. Denied requests write `kernel/outbound.denied`; allowed requests write `kernel/outbound.request` with redacted audit.

3. **Request/response audit**
    - Every outbound request needs principal, package id, capability id, provider family, route id, redaction state, and cost/usage placeholders.
    - Raw prompts/responses require redaction policy before audit persistence.
    - **Phase S2 progress**: `OutboundAuditRecord` captures principal, package_id, capability_id, destination_host, method, purpose, redaction_state, secret_refs_used, usage/cost placeholders, status/error. `RedactionState` enum: `not_captured`, `redacted`, `policy_ref`, `unsafe_blocked`, `explicitly_approved`. Default is `redacted` — raw body/header/prompt/response never saved. Inspectable via `kernel.outbound.audit`.

4. **Streaming and cancellation**
   - Streaming chunks need a public protocol shape.
   - Cancellation/timeout behavior must be deterministic and tested.

5. **Usage accounting**
   - Provider usage units must normalize without losing provider-specific details.
   - Cost estimates must be marked estimated unless provider-confirmed.

6. **Provider error taxonomy**
   - Authentication, rate limit, quota, timeout, model not found, malformed request, and provider outage errors must map to stable package-level diagnostics.

7. **Data redaction and approval**
    - Assistant-mediated inference must be approval-gated when sending user or project data outside the host boundary.
    - Redaction policies must be inspectable through public surfaces.
    - **Phase S1/S2 progress**: Conservative redaction scanner for known secret field names and value patterns in trusted paths (proposal payloads, asset metadata). Outbound audit records use `RedactionState` enum with `not_captured`/`redacted`/`policy_ref`/`unsafe_blocked`/`explicitly_approved`. Content/description/title/reason fields are excluded from value-pattern scanning to avoid false positives.

## Deferred capabilities

Future inference packages may add:

- generate request planning;
- non-streaming generation;
- streaming generation;
- embedding calls;
- provider model discovery;
- tool-call mediation;
- usage reports;
- safety/redaction previews.

None of these are part of Model Connectivity Kit Alpha.
