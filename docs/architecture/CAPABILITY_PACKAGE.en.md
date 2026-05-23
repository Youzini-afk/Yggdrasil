# Capability Package Specification

> [English](./CAPABILITY_PACKAGE.en.md) · [中文](./CAPABILITY_PACKAGE.md)

A capability package is Yggdrasil's unit of distribution and execution. Anything outside the kernel ships as a package.

This document describes how a package identifies itself, loads, and interacts with the kernel and other packages. Every package follows the same rules, regardless of origin.

## Equality rule

Official, third-party, in-process, subprocess, WASM, and remote packages share one manifest format, one lifecycle, one capability fabric, and one permission system.

There is no private API. Anything an official package can do, any package can do.

## Manifest

A package is described by a manifest. It is a serializable document that must conform to a published schema.

```yaml
schema_version: 1

id: org/name              # globally unique, namespaced
version: 0.1.0            # semver
display_name: ...
description: ...
author: ...
license: ...

entry:
  kind: rust_inproc | subprocess | wasm | remote

  # kind: rust_inproc
  crate: path or registry coordinate
  symbol: register_fn
  abi_version: 1

  # kind: subprocess
  command: [executable, args...]
  env: { ... }
  transport: jsonrpc-stdio | jsonrpc-tcp

  # kind: wasm
  module: path or url
  abi_version: 1
  memory_limit_mb: 64

  # kind: remote
  endpoint: https://... or wss://...
  auth: { scheme: bearer | mtls | none, ... }

provides:
  - id: org/name/capability
    version: 0.1.0
    input_schema: <jsonschema or ref>
    output_schema: <jsonschema or ref>
    streaming: false
    side_effects: [event_append, network, filesystem, package_call, ...]
    description: ...

consumes:
  - id: other-org/cap
    version: ^0.2

contributes:
  schemas:
    - id: org/name/event/foo
      schema: <jsonschema>
  hooks:
    - extension_point: kernel/v1/event.after_append
      handler: handle_event
      timing: async
  assets:
    - id: org/name/asset/...
      mime: ...
      source: ...
  extension_points:
    - id: org/name/lifecycle.before_step
      payload_schema: <jsonschema>
      timing: sync | async
      modifiable: true
      short_circuit: true
  surfaces:
    - id: org/name/entry
      version: 0.1.0
      slot: experience_entry        # | home_card | play_renderer | forge_panel | asset_editor | assistant_action
      title: ...
      description: ...
      capability_id: org/name/launch
      activation:
        launch_capability_id: org/name/launch
        session_template:
          labels: [...]
          metadata: { ... }
        input_schema: <jsonschema>
      required_permissions:
        - permission: events.read
          scope: session
          reason: render the play surface
          risk: low                 # | medium | high
      approval_policy: none         # | user_approval | fork_then_approve
      metadata: { ... }

permissions:
  network:
    hosts: [api.example.com] | none | any
  filesystem:
    paths: [./data] | none
  events:
    append: true
    read: true
  packages:
    call: [other-org/*]
  declared_side_effects: [user-data-read, llm-inference, ...]

sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 128
  wall_clock_ms: 30000
```

The kernel rejects manifests that fail schema validation. It also refuses to load a package that requests permissions outside the host's policy.

## Entry forms

All four entry forms are first-class. The choice is an implementation detail.

### rust_inproc

Loaded as a Rust crate or shared library compiled against the kernel's package ABI. It is fast, has no IPC cost, and gives the best performance. Trust level is highest. Crashes can affect the host; the sandbox is the host itself.

### subprocess

The kernel spawns a child process and speaks JSON-RPC over stdio or a local socket. It is language-agnostic, and crashes are isolated. Performance is bounded by IPC.

### wasm

The kernel runs the package inside a WASM host with declared memory and CPU caps. Isolation is strong. Language choice is limited to WASM-targetable languages, and performance is bounded by WASM and ABI marshalling.

### remote

The package runs anywhere reachable over HTTP or WebSocket. The connection is authenticated. This fits hosted services and external systems that participate as packages.

A package may declare alternative entries. For example, it can offer `rust_inproc` plus a `subprocess` fallback, and let the host pick by policy.

## Lifecycle

```text
discovered  -> kernel sees the manifest
loading     -> manifest validated, sandbox prepared
starting    -> entry point booted, kernel handshake
ready       -> capabilities and hooks registered, accepting calls
degraded    -> reachable but reporting reduced ability
stopping    -> graceful shutdown signal sent
stopped     -> resources released
unloaded    -> manifest no longer active in the host
```

Each state transition emits a kernel event.

## Capability contract

A capability is identified by `id` and `version`. Calls are typed by `input_schema` and `output_schema`. They may stream.

A consumer requests a capability by id and version constraint. The kernel selects a provider based on:

1. Active package set in the session scope.
2. Declared precedence rules in the session/profile. The kernel has no default precedence; the policy is configured by the host or a routing package.
3. Compatibility of versions.

There is no implicit "official wins" rule.

If two packages provide the same capability id and the host has not configured precedence, the kernel reports an ambiguous-route error and refuses the call.

## Hook contract

A package may subscribe to extension points. They can be defined by the kernel or by packages. The subscription declares timing and whether the handler may mutate or veto.

The kernel dispatches hooks according to their declared semantics. Subscribers run in declared order; ties use subscriber precedence configured by the host.

See `EXTENSION_POINTS.md` for the kernel-emitted point set and the contract.

## Permissions and sandbox

The manifest is a contract with the host. The kernel enforces it on every operation:

- An undeclared event append is refused.
- An undeclared network call is refused.
- An undeclared cross-package invocation is refused.
- A capability that exceeds its declared `side_effects` is refused.

The host may layer additional policy on top, such as deny-lists, quotas, and audit. Packages cannot bypass it.

## Distribution

A package distribution includes the manifest and the entry artifact:

- For `rust_inproc`: source crate or precompiled `cdylib` matching the host ABI version.
- For `subprocess`: an executable for the target platform plus the manifest.
- For `wasm`: a `.wasm` module plus the manifest.
- For `remote`: just the manifest with the endpoint.

A package registry is out of scope for the kernel.v1. Hosts and tools may build registries on top.

## Versioning

`version` follows semver. `schema_version` of the manifest format is independent of package version.

A breaking ABI change in `rust_inproc` is signaled by a new `abi_version` in the entry. Hosts refuse to load mismatched ABI versions.

## Identity

Package id is namespaced. The kernel does not own the namespace; conventions and registries do.

The kernel only enforces uniqueness inside one host instance.
