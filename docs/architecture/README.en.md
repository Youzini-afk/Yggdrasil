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
- [`HOST_PROJECT_AUTHORITY.md`](HOST_PROJECT_AUTHORITY.en.md) — project-scoped resources, authenticated context, session binding, and authorization audit
- [`DURABLE_DEPLOYMENT_CONTROLLER.md`](DURABLE_DEPLOYMENT_CONTROLLER.en.md) — desired/observed state, idempotent operations, safe activation, and recovery
- [`TARGET_AGENT_PROTOCOL.md`](TARGET_AGENT_PROTOCOL.en.md) — remote target identity, typed operations, artifact/secret, and tunnel boundaries
- [`OPERATIONS_DATA_RELEASE.md`](OPERATIONS_DATA_RELEASE.en.md) — migration, backup, health, diagnostics, upgrade, and supply-chain gates
