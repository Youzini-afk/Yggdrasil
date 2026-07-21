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

Registry `0.4.0` currently publishes 36 identity aliases:

| Canonical | Legacy alias | Owner |
|---|---|---|
| `host.info` | `kernel.v1.host.info` | `host` |
| `host.project.{list,get,start,stop,status}` | `kernel.v1.project.*` | `host` |
| `host.target.{list,status,register,unregister}` | `kernel.v1.target.*` | `host` |
| `host.exec.{start,stop,status,logs,list}` | `kernel.v1.exec.*` | `host` |
| `host.port.{lease,release,status,list}` | `kernel.v1.port.*` | `host` |
| `host.proxy.{register,unregister,status,list}` | `kernel.v1.proxy.*` | `host` |
| `host.surface.bundle.resolve` | `kernel.v1.surface.resolve_bundle` | `host` |
| `shell.contribution.{list,describe}` | `kernel.v1.surface.contribution.*` | `shell` |
| `change.proposal.{create,get,list,approve,reject,apply}` | `kernel.v1.proposal.*` | `protocol` |
| `projection.{register,rebuild,get,list}` | `kernel.v1.projection.*` | `protocol` |

The `*` and `{...}` notation is documentation shorthand; every suffix is registered explicitly.
Until migrated, every other method keeps its existing `kernel.v1.*` ID as its canonical ID. New
aliases must be registered centrally; dispatchers, clients, and transports must not add string
special cases.

Phase 3 changes only ownership and namespace. Payloads, permissions, events, and handlers remain
unchanged. In particular, `change.proposal.*` still uses the existing `ProposalRecord`; it does not
pretend to provide the Intent, ChangeSet, Commit, or EffectReceipt primitives introduced in Phase 5.

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
- The advertised profiles are currently `ygg.contract.default/v1`, `ygg.shell.default/v1`, and
  `kernel.v1`. Shell Default requires the published host, protocol, and shell layer versions.
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
- generation rejects duplicate canonical/alias wire IDs, TypeScript/Rust function names, and
  OpenAPI operation IDs, and validates each alias target and replacement.

Schemas, SDKs, and OpenAPI are regenerated together; generated artifacts are not edited manually.

## Deprecated aliases and diagnostics

Registry `0.4.0` begins the first measured deprecation window:

| Legacy alias | Replacement | Replacement maturity | Deprecated in | Supported through |
|---|---|---|---|---|
| `kernel.v1.host.info` | `host.info` | Candidate | `ygg.contract.registry@0.4.0` | `ygg.contract.registry@0.5.0` |
| `kernel.v1.target.list` | `host.target.list` | Candidate | `ygg.contract.registry@0.4.0` | `ygg.contract.registry@0.5.0` |

The old and canonical IDs still reach the same handler and return the same method result. HTTP RPC,
host stdio, and subprocess reverse stdio add an optional top-level `diagnostics` array when a
tracked deprecated alias is requested. The ad-hoc `GET /kernel/v1/host.info` route exposes the same
policy through `x-yggdrasil-contract-*` response headers and a `Link` to `/rpc`. The replacement
header value is a canonical method ID, not a URL; invoke it with `POST /rpc`. Diagnostics are
advisory and do not alter the method payload or error mapping, including when contract selection is
structurally invalid but the requested legacy method ID can still be recovered.

Run a read-only migration preview with:

```sh
ygg contract migrate PATH --json
```

By default the tool migrates only aliases with a published deprecation window. Add `--all-aliases`
to opt into proactive migration of every registered alias, and add `--write` only after reviewing the
preview. Replacements require whole contract-ID boundaries; the scanner accepts a conservative
source/Markdown extension allowlist and reports every unsupported, non-UTF-8, or oversized file it
skips, plus every excluded symlink or build/dependency/vendor directory. Writes use same-directory
staging plus atomic replacement, and previously applied files are rolled back if a later write fails.
Excluded paths are never followed. Web is the first real client migrated with `--all-aliases`: its protocol client, surface
bridge, bundle resolver, tests, and guide now use canonical IDs.
