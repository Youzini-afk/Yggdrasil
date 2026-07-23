# Architecture

> [English](./README.en.md) · [中文](./README.md)

The kernel + packages + projects three-tier architecture, capability package contract, extension points, event model, and lifecycles.

## Platform stance

- [`VISION.md`](VISION.en.md) — what the platform is for
- [`ARCHITECTURE.md`](ARCHITECTURE.en.md) — kernel + packages + projects layering
- [`PLATFORM_KERNEL.md`](PLATFORM_KERNEL.en.md) — what the kernel does and does not do
- [`CONSTITUTION_V2.md`](CONSTITUTION_V2.en.md) — candidate v2 constitution: long-term layers, invariants, and anti-rigidity constraints; it does not yet replace the current contract

## Capability package contract

- [`CAPABILITY_PACKAGE.md`](CAPABILITY_PACKAGE.en.md) — package contract
- [`EXTENSION_POINTS.md`](EXTENSION_POINTS.en.md) — hook contract
- [`EVENT_MODEL.md`](EVENT_MODEL.en.md) — opaque event log model
- [`RUNTIME_LIFECYCLE.md`](RUNTIME_LIFECYCLE.en.md) — kernel-side lifecycles

## Upstream integration boundaries

- [`PI_INTEGRATION.md`](PI_INTEGRATION.en.md) — absorption boundary for the pi agent framework

## Host control planes

- [`HOST_DEVELOPMENT_CONTROL_PLANE.md`](HOST_DEVELOPMENT_CONTROL_PLANE.en.md) — controlled source changes, verification, promotion, and recovery
- [`HOST_REMOTE_ACCESS.md`](HOST_REMOTE_ACCESS.en.md) — root/device identities, scopes, HTTPS pairing, and explicit application-route exposure
