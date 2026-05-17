# Model Inference Prerequisites

> [English](./MODEL_INFERENCE_PREREQUISITES.md) · [中文](./MODEL_INFERENCE_PREREQUISITES.zh-CN.md)

Model Connectivity Kit Alpha deliberately stops before real model execution. A future `official/model-inference-lab` or equivalent package family must not start until these prerequisites are specified and covered by conformance.

## Required platform contracts

1. **Secret resolution**
   - Profile assets may reference secrets, but raw secrets must not appear in events, projections, logs, UI, or assistant proposals.
   - Hosts need a public secret-reference capability or policy surface.

2. **Network permission**
   - Packages need explicit network permissions by destination, method, and purpose.
   - No package should infer network permission from being official.

3. **Request/response audit**
   - Every outbound request needs principal, package id, capability id, provider family, route id, redaction state, and cost/usage placeholders.
   - Raw prompts/responses require redaction policy before audit persistence.

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
