# Play-creation model

> [English](./PLAY_CREATION_MODEL.en.md) · [中文](./PLAY_CREATION_MODEL.md)

This document fixes Yggdrasil's product stance. Yggdrasil is not a chat tool, not a game engine, not a Tavern compatibility layer, and not a developer workbench. It's a play-creation platform, meant to make AI-native experiences possible that didn't exist before — and to let the people playing them inspect, modify, and fork them.

This stance is what the kernel, the public protocol, the official packages, the web shell, and the SDKs all serve. When a future feature seems to clash with the stance, the stance wins.

## The play-creation premise

Most AI-native creation tools today split people in two: players who consume a finished experience, and developers who build it. Yggdrasil refuses that split.

A player on Yggdrasil can:

- start a session;
- inspect what's happening;
- ask an assistant to change something;
- fork the session to try another path;
- swap one capability package for another;
- save their version and share it.

A creator on Yggdrasil can:

- write a package that loads on any host;
- declare entries, capabilities, hooks, and surfaces;
- debug live with the same protocol the player uses;
- watch the play-creation loop run on top of their package — no separate "developer mode."

The substrate is the same in both directions. There is no "player edition" and "developer edition" of Yggdrasil. There's one host speaking the public protocol, and the rest is a choice of packages.

## Three first-class surfaces

The platform organizes itself around three surfaces. The kernel knows their slot names; the kernel does not know what they mean.

### Home / Play

A console-style launcher and play surface. It discovers playable content from `experience_entry` descriptors that packages declare, and renders package-declared `play_renderer` surfaces in the session. Home / Play is where most people spend most of their time.

Home isn't a store, and it isn't a router. It's a surface that asks the public protocol "what's launchable here right now?" and trusts packages to answer.

### Forge

The agentic creation workspace. It honestly exposes the substrate: events, capabilities, assets, projections, branches, proposals, surfaces, packages, hooks, permissions. It hosts package-declared `forge_panel` surfaces, so packages can put their own creation or inspection panels next to the generic inspectors.

Forge is where a play-creator becomes a creator-creator without leaving the platform. Visual editors, node editors, prompt editors, lorebooks, world maps — those tools live in Forge as Forge editor modes contributed by packages, not as kernel features.

### Assist

A cross-mode assistant drawer. In Play, it offers small live tweaks and proposals. In Forge, it does deeper work — proposing operations, drafting packages, explaining diffs, suggesting changes. In both modes, every change goes through `kernel.v1.proposal.*` and is approved before it lands.

Assist is a thin client of the proposal lifecycle. It is not a privileged path for changes. A third-party assistant package can replace `official/assistant-lab` and run the same way.

## The creator flow

The play-creation loop runs on the existing substrate. End to end:

1. Home discovers `experience_entry` surfaces over the public protocol.
2. A player launches an experience.
3. The kernel opens a session bound to the package set that experience needs.
4. The package writes its own events and drives its own `play_renderer`.
5. The player asks Assist to change something.
6. Assist (also a package) calls `kernel.v1.proposal.create` with generic operations.
7. The player reviews the proposal and approves it.
8. The kernel applies the approved operations and writes `kernel/v1/proposal.applied`.
9. The player can fork the session at a sequence number to try another path.
10. The player can open Forge to inspect events, assets, projections, branches.
11. The player can edit a package or composition through a Forge editor.
12. The loop continues.

The kernel never invents domain semantics for any of these steps. Semantics belong to packages. The loop works because packages can declare their own surfaces, propose their own operations, and own their own events — while the kernel mediates generically.

## What the platform provides — and doesn't

The platform provides:

- a content-free kernel;
- a manifest model for packages;
- a permission and principal model for humans, assistants, packages, and hosts;
- one public protocol everyone uses;
- generic surface contributions for Home / Play, Forge, asset editors, and assistant actions;
- a generic proposal / approval lifecycle for any change;
- a generic asset, branch, and projection substrate;
- official foundation packages that demonstrate, not privilege.

The platform does not provide:

- a chat experience, or any other genre;
- a model-provider abstraction;
- a memory model, retrieval strategy, or director;
- a SillyTavern compatibility layer;
- an external game-engine bridge;
- a favored visual editor or asset editor;
- a marketplace.

Each of these is welcome as a package. None is welcome as the kernel.v1.

## Stance on Tavern, agents, and external engines

SillyTavern resources, agent loops, and external-engine bridges are all valuable — but they belong to package families, not the platform family.

When they arrive, they will be ordinary packages, bound by the same manifests, the same fabric, the same permissions, and the same sandbox rules as any third-party package. They will not get kernel privilege. The play-creation loop runs on top of them the same way it runs on top of a tiny fixture experience: discover, launch, propose, approve, apply, fork.

If one day a Tavern-shaped runtime ships as an official package, a third-party world simulator package must be able to coexist with it in the same session. If it can't, the bug is in the kernel, not the third-party package.

## Stance on radical creation

The goal isn't to ship a better Tavern. The goal is to let experiences exist that the platform's authors didn't foresee — and to let the players who try those experiences fork, inspect, modify, and share what they find.

The substrate already biases toward this. Events are append-only and the kernel owns ordering. Branches are first-class. Proposals are auditable. Surfaces are descriptors, not hardcoded UI. Packages are equal regardless of origin or entry form.

When a feature decision makes radical creation harder — by privileging an official path, by hiding state from inspection, by forcing a single shape — the charter wins, and the feature gives way.

## Stance on "release"

There is no "1.0 chat experience" target. The platform's release shape is:

- **Foundation Alpha** — the substrate is content-free and trustworthy (reached).
- **Playable Experience Alpha** — at least one experience runs end to end on the substrate, replaceable, forkable, assistant-aided.
- **Authoring Beta** — third parties can ship packages on equal footing with official ones.
- **Substrate v1** — the substrate stops moving fast and commits to public protocol stability.

Anything past Substrate v1 is product scope. The platform never owns it.
