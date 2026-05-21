# Tavern Compatibility (deferred)

> [English](./TAVERN_COMPAT.en.md) · [中文](./TAVERN_COMPAT.md)

This document is reserved for a future capability package family. That family will import SillyTavern resources and reproduce enough Tavern behavior for community content to run on Yggdrasil. It is not on the near-term path.

## Position

Tavern compatibility is not part of the kernel. The kernel does not understand character cards, world books, presets, prompt rendering, or any other content-shaped concern.

When Tavern compatibility is built, it will ship as one or more capability packages. It will be governed by the same manifest, fabric, permission, and sandbox rules as any third-party package. It will receive no kernel privileges.

## Likely shape (sketch only)

A future Tavern package family might include these separate packages:

- A resource importer that parses Character Card V2, PNG-embedded metadata, world books, presets, and chat history.
- A native projection package that converts those into package-defined assets and events.
- A behavior layer that reproduces Tavern-like prompt rendering and lorebook activation, used by an official conversational runtime package or by Tavern-shaped runtime packages.
- An extension shim, where applicable, mapping Tavern extension concepts onto Yggdrasil capabilities.

The kernel will only see packages that declare event kinds, capabilities, and assets in their own namespaces. They are no different from any other package.

## Lossless import principle (carried forward)

When the work happens, imported resources keep their original payload alongside any native projection. Old schemas should not define what the platform can express, but they also should not be destroyed on import.

```text
original_payload   the original SillyTavern data, untouched
native_projection  package-defined views derived from it
```

This principle belongs to the importer package, not to the kernel.

## Non-goals for the kernel

The kernel will never:

- ship a SillyTavern parser,
- model character cards or world books,
- hardcode `{{char}}` / `{{user}}` substitution,
- offer Tavern-specific hooks or methods,
- treat Tavern packages differently from any other package.

## Status

Tavern compatibility is deferred until at least one playable conversational/runtime capability package exists on Yggdrasil. At that point there will be a real consumer for Tavern-shaped content.

The platform substrate it would need is already in place: packages, events, capabilities, hooks, permissions, surface contributions, proposals, assets, branches, and projections. Tavern compatibility, when built, can run entirely as packages with no kernel changes.

Until then, this document only fixes the position: Tavern compatibility is a future package family, not a platform layer.
