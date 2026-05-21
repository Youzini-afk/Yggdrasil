# Specifications

> [English](./README.en.md) · [中文](./README.md)

Executable alpha contracts and the hostile conformance roadmap. These docs are backed by code and tests — not aspirational specs.

- [`KERNEL_V0_ALPHA_CONTRACT.md`](KERNEL_V0_ALPHA_CONTRACT.en.md) — kernel v0 alpha contract matrix: protocol methods, status, streaming flags, namespaces, red lines
- [`CONFORMANCE_MATRIX.md`](CONFORMANCE_MATRIX.en.md) — hostile conformance case inventory, indexed by tag and domain

Run the full suite:

```bash
cargo run -p ygg-cli -- conformance
```
