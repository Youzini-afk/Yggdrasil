# Contract Registry and Explicit Negotiation (Experimental)

> [English](./CONTRACT_REGISTRY.en.md) · [中文](./CONTRACT_REGISTRY.md)

This document describes the first executable compatibility mechanism for the layered contract
migration. It is Experimental: it does not claim that Constitution v2 is Stable and does not
change existing `kernel.v1.*` payload semantics.

## One resolution boundary

Before a permission gate or handler runs, the runtime always:

1. validates the optional contract selection;
2. resolves a canonical ID or alias through the central registry;
3. applies the request adapter;
4. invokes the single `KernelMethod` handler;
5. applies the response adapter.

HTTP RPC, host stdio, in-process calls, and subprocess reverse stdio share this logic. An alias
does not create a second handler, principal, permission, or audit path.

## Registry shape

Each `ContractMethod` advertises:

- `canonical_id` and `aliases`;
- `owner_layer` and `maturity`;
- request and response schema URIs;
- request and response adapters;
- introduced, deprecated, and replacement metadata;
- current implementation status and streaming metadata.

The initial identity aliases are:

| Canonical | Legacy alias | Owner |
|---|---|---|
| `host.info` | `kernel.v1.host.info` | `host` |
| `host.target.list` | `kernel.v1.target.list` | `host` |

Until migrated, every other method keeps its existing `kernel.v1.*` ID as its canonical ID. New
aliases must be registered centrally; dispatchers, clients, and transports must not add string
special cases.

## Explicit negotiation

The RPC envelope may include an optional field:

```json
{
  "id": "request-1",
  "method": "host.info",
  "params": {},
  "contract": {
    "profile": "ygg.contract.default/v1",
    "versions": [
      { "layer": "host", "version": "0.1.0" }
    ]
  }
}
```

- Omitting `contract` selects the `kernel.v1` legacy profile for old clients.
- Once a client explicitly requests a profile or layer version, the host must satisfy it exactly.
- Unknown profiles, layers outside the profile, and version mismatches return
  `kernel/v1/error/unsupported_contract` with a structured reason.
- The host never silently falls back to a weaker profile and never invokes the business handler
  after negotiation fails.

## `host.info`

The existing `protocol_version`, `methods`, and `supported_transports` fields remain unchanged.
The following fields are additive and optional:

- `contract_registry_version` and `default_profile`;
- `layers`, `versions`, `profiles`, and `maturity`;
- `aliases` and `contract_methods`.

Old SDKs can ignore these fields, while new SDKs must allow them to be absent when connected to an
older host.

## SDKs

The generator reads `x-yggdrasil-contract` metadata from every method schema:

- existing source-level method names invoke the canonical wire ID;
- each legacy wire ID gets an explicit `legacyKernelV1...` / `legacy_kernel_v1_...` wrapper;
- a negotiated client is enabled only when its transport can carry contract selection, so a
  requirement is never silently ignored.

Schemas, SDKs, and OpenAPI are regenerated together; generated artifacts are not edited manually.

