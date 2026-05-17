# pi Integration (deferred)

> [English](./PI_INTEGRATION.en.md) · [中文](./PI_INTEGRATION.md)

This document is reserved for a future capability package family that integrates the `pi` agent framework with Yggdrasil. It is not on the near-term path.

## Position

pi is not part of the kernel. The kernel ships zero opinion about agents, planners, proposals, memory curation, or any other content-shaped concern.

When pi integration is built, it will ship as one or more capability packages, governed by the same manifest, fabric, permission, and sandbox rules as any third-party package. It will receive no kernel privileges.

## Likely shape (sketch only)

The platform contract gives every package the tools it would need:

- Subscribe to events via `kernel.event.subscribe`.
- Append package-namespaced events under the writer's own kind set (e.g., `pi/<...>/proposal.created`).
- Provide capabilities other packages can invoke (e.g., `pi/<...>/curate`, `pi/<...>/extract`).
- Define its own extension points so other packages can subscribe to pi-internal stages.
- Declare permissions, sandbox limits, and side effects in manifest.

A "proposal then commit" pattern, if pi adopts it, is implemented as ordinary events and capability calls between pi packages and other packages. The kernel does not need to know about it.

## Non-goals for the kernel

The kernel will never:

- model "agents" as a first-class concept,
- model "proposals" as a first-class concept,
- model memory taxonomy,
- offer pi-specific hooks or methods,
- treat pi packages differently from any other package.

## Status

pi integration is deferred until the play-creation platform substrate is consolidated. The substrate it would need — events, capabilities, hooks, permissions, surfaces, and the proposal/approval lifecycle — is now in place, so when integration begins it can ship as ordinary capability packages with no kernel changes. Until then, this document only fixes the position: pi is a future package family, not a platform layer.
