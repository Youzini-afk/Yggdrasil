# Specifications

> [English](./README.en.md) · [中文](./README.md)

Executable v1 contracts and the hostile conformance roadmap. These docs are backed by code and tests — not aspirational specs.

- [`KERNEL_V1_CONTRACT.md`](KERNEL_V1_CONTRACT.en.md) — kernel v1 public contract: 80 methods, 57 events, capability handles, Path A / Path B, SDKs, and conformance
- [`CONFORMANCE_MATRIX.md`](CONFORMANCE_MATRIX.en.md) — hostile conformance case inventory, indexed by tag and domain
- [`v1/EVENT_KIND_REGISTRY.md`](v1/EVENT_KIND_REGISTRY.en.md) — v1 event kind registry
- [`v1/ERROR_CODES.md`](v1/ERROR_CODES.en.md) — v1 error codes
- [`v1/VERSIONING.md`](v1/VERSIONING.en.md) — v1 additive-only versioning strategy
- [`v1/schemas/`](v1/schemas/) — 144 JSON Schemas (methods / events / top-level), the SDK source of truth

Run the full suite:

```bash
cargo run -p ygg-cli -- conformance
```
