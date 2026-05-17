# Model Connectivity Kit

> [English](./MODEL_CONNECTIVITY_KIT.md) · [中文](./MODEL_CONNECTIVITY_KIT.zh-CN.md)

Model Connectivity Kit Alpha is Yggdrasil’s first model-provider infrastructure layer. It prepares provider profiles and route plans, but it does not send data to external providers.

## Included packages

- `official/model-connector-lab`
  - `describe_families`
  - `validate_profile`
  - `mask_secret`
  - `discovery_plan`
  - `compat_report`
- `official/model-routing-lab`
  - `define_binding`
  - `resolve_binding`
  - `preview_routes`
  - `params_normalize`
  - `compat_report`

## Alpha safety boundary

Alpha is intentionally no-network and no-inference:

- discovery is a plan, not a live provider query;
- validation is structural, not credential verification;
- `secret_ref` is accepted, raw secrets are rejected;
- route resolution chooses from explicit bindings, not a hidden global route;
- params normalization keeps provider-specific options namespaced;
- outputs include provenance and `network_performed: false` or `inference_performed: false` where relevant.

## Typical flow

1. Describe provider families with `official/model-connector-lab/describe_families`.
2. Validate a redaction-safe profile with `official/model-connector-lab/validate_profile`.
3. Generate a no-network model discovery plan with `official/model-connector-lab/discovery_plan`.
4. Define consumer-slot bindings with `official/model-routing-lab/define_binding`.
5. Resolve deterministic routes with `official/model-routing-lab/resolve_binding`.
6. Normalize generation-like params with `official/model-routing-lab/params_normalize`.

Persistence should still be done through public asset/proposal protocol operations. The model labs do not gain special write privileges.

## TavernHeadless reference

`integrations/tavern-headless/model-connectivity-map.yaml` tracks provider/profile/instance behavior reviewed from TavernHeadless. The map is a reference ledger only. Yggdrasil does not create `tavern-*` model packages.

## Deferred inference

Real model calls belong to a future package family, likely `official/model-inference-lab`, after Yggdrasil specifies secret resolution, network permission, request/response audit, streaming/cancel policy, usage accounting, provider errors, and redaction rules.
