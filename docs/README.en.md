# Yggdrasil Documentation

> [English](./README.en.md) · [中文](./README.md)

Topic-grouped navigation for the developer docs. Every doc has both English and Simplified Chinese versions; the bilingual blockquote at the top of each file switches between them.

## Stance and status

- [`CHARTER.md`](CHARTER.en.md) — permanent principles
- [`ALPHA_STATUS.md`](ALPHA_STATUS.en.md) — living snapshot of done / partial / deferred
- [`../BUILDING.md`](../BUILDING.md) — Rust, web, Tauri desktop, and release build notes
- [`product/`](product/README.en.md) — play-creation product stance and experience-led platform direction

## Architecture and protocol

- [`architecture/`](architecture/README.en.md) — kernel + packages layering, capability contract, extension points, event model, lifecycles
- [`protocol/`](protocol/README.en.md) — public protocol spec
- [`spec/`](spec/README.en.md) — executable v1 contract matrix, hostile conformance roadmap, schemas

## Authoring

- [`guides/`](guides/README.en.md) — capability package authoring guides, grouped by domain (foundation / agent / model / inference / experience / memory / storage / external projects / sharing)
- [`guides/CAPABILITY_HANDLES.md`](guides/CAPABILITY_HANDLES.en.md) — v1 capability handles, attenuation, revoke, and effect audit
- [`guides/CONFORMANCE_KIT.md`](guides/CONFORMANCE_KIT.en.md) — third-party package v1 conformance kit
- [`guides/PATH_B_SELF_CONTAINED.md`](guides/PATH_B_SELF_CONTAINED.en.md) — `entry.contract: "none"` self-contained path
- [`guides/SURFACE_HOSTING.md`](guides/SURFACE_HOSTING.en.md) — `clients/web` iframe SurfaceHost and third-party web surface bundle hosting

## Performance and roadmap

- [`performance/`](performance/README.en.md) — performance baseline, conformance feedback loop, code health
- [`roadmap/`](roadmap/README.en.md) — current and upcoming phases, model inference prerequisites
- [`tavern/`](tavern/README.en.md) — how Yggdrasil relates to YdlTavern, the SillyTavern-compatible integration project

## Shortest path by intent

| If you want to | Read first |
|---|---|
| Understand the platform stance | [`CHARTER.md`](CHARTER.en.md) → [`architecture/VISION.md`](architecture/VISION.en.md) → [`product/PLAY_CREATION_MODEL.md`](product/PLAY_CREATION_MODEL.en.md) |
| Understand the architecture | [`architecture/ARCHITECTURE.md`](architecture/ARCHITECTURE.en.md) → [`architecture/PLATFORM_KERNEL.md`](architecture/PLATFORM_KERNEL.en.md) → [`architecture/CAPABILITY_PACKAGE.md`](architecture/CAPABILITY_PACKAGE.en.md) |
| Use the public protocol | [`protocol/PROTOCOL_V0.md`](protocol/PROTOCOL_V0.en.md) → [`spec/KERNEL_V1_CONTRACT.md`](spec/KERNEL_V1_CONTRACT.en.md) |
| Write your first package | [`guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](guides/PACKAGE_AUTHORING_WALKTHROUGH.en.md) |
| Host third-party web surfaces | [`guides/SURFACE_HOSTING.md`](guides/SURFACE_HOSTING.en.md) |
| Build web / desktop / release | [`../BUILDING.md`](../BUILDING.md) |
| See current status | [`ALPHA_STATUS.md`](ALPHA_STATUS.en.md) |
| See what's next | [`roadmap/NEXT_STEPS.md`](roadmap/NEXT_STEPS.en.md) |
