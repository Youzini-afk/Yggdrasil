# Play-Creation Model

> [English](./PLAY_CREATION_MODEL.en.md) · [中文](./PLAY_CREATION_MODEL.md)

This document fixes the product stance of Yggdrasil. The platform is not a chat tool, not a game engine, not a Tavern compatibility layer, and not a developer workbench. It is a play-creation platform whose purpose is to make AI-native experiences that did not exist before, and to keep them open to inspection, modification, and forking by the people who play them.

The stance pinned here is what the kernel, the public protocol, the official packages, the web shell, and the SDKs collectively serve. When a future feature appears to push against it, the stance wins.

## The play-creation premise

Most AI-native creative tools today divide their users into two roles: a player who consumes a finished experience, and a developer who builds it. Yggdrasil refuses that split.

A player on Yggdrasil can:

- start a session,
- inspect what is happening,
- ask an assistant to change something,
- fork the session and try a different shape,
- replace one package with another,
- save what they made and share it.

A creator on Yggdrasil can:

- write a package that is loadable on any host,
- declare entry points, capabilities, hooks, and surfaces,
- debug live against the same protocol the player uses,
- watch the play-creation loop happen on their package without a separate "developer mode."

The substrate is the same in both directions. There is no "developer build" of Yggdrasil and no "player build." There is a host running the public protocol, and the rest is package decisions.

## Three first-class surfaces

The platform organizes itself around three surfaces. The kernel knows their slot names; the kernel does not know their meaning.

### Home / Play

Console-like launcher and play surface. It discovers what is playable from package-declared `experience_entry` surfaces and renders package-declared `play_renderer` surfaces in a session. Home/Play is where most time is spent for most users.

Home is not an app store and not a router. It is a surface that asks the public protocol "what is launchable here right now?" and trusts packages to answer.

### Forge

Agentic creation workspace. It exposes the substrate honestly: events, capabilities, assets, projections, branches, proposals, surfaces, packages, hooks, permissions. It hosts package-declared `forge_panel` surfaces so packages can ship their own creation/inspection panels alongside the generic inspectors.

Forge is the place where a player-creator becomes a creator-creator without leaving the platform. Visual editors, node editors, prompt editors, lorebooks, world maps, and similar tools belong inside Forge as Forge editor modes contributed by packages, not as kernel features.

### Assist

Cross-mode assistant drawer. In Play, it offers small live edits and proposals. In Forge, it does deeper work — proposing operations, drafting packages, narrating diffs, suggesting changes. In both modes, every change goes through `kernel.proposal.*` and is approved before it lands.

Assist is a thin client of the proposal lifecycle. It is not a privileged mutation path. A third-party assistant package can replace `official/assistant-lab` and behave the same way.

## Player-creator flow

The play-creation loop runs on existing substrate. It looks like this end-to-end:

```text
Home discovers experience_entry surfaces over the public protocol.
Player launches an experience.
Kernel opens a session bound to the package set the experience needs.
Package writes its own events, drives its own play_renderer surface.
Player asks Assist to change something.
Assist (a package) calls kernel.proposal.create with generic operations.
Player reviews the proposal and approves it.
Kernel applies approved operations and writes kernel/proposal.applied.
Player optionally forks the session at a sequence to try another path.
Player optionally opens Forge to inspect events, assets, projections, branches.
Player optionally edits a package or composition through Forge editor modes.
Cycle continues.
```

The kernel never invents domain semantics for any of these steps. The semantics belong to packages. The loop only works because packages can declare the surfaces they contribute, the operations they propose, and the events they own — and because the kernel mediates them generically.

## What the platform does and does not provide

The platform provides:

- a content-free kernel,
- a manifest model for packages,
- a permission and principal model for humans, assistants, packages, and hosts,
- a public protocol that everyone uses,
- generic surface contributions for Home/Play, Forge, asset editors, and assistant actions,
- generic proposal/approval lifecycle for any change,
- generic asset, branch, and projection substrate,
- official foundation packages that demonstrate, never privilege.

The platform does not provide:

- a chat experience or any other genre,
- a model provider abstraction,
- a memory model, retrieval policy, or director,
- a SillyTavern compatibility layer,
- an external game engine bridge,
- a blessed visual editor or asset editor,
- a marketplace.

Each of those is welcome as a package. None is welcome as kernel.

## Stance toward Tavern, agents, engines

SillyTavern resources, agent loops, and external engine bridges are valuable, but they are package families, not platform families.

When they arrive, they will be ordinary capability packages governed by the same manifest, fabric, permission, and sandbox rules as any third-party package. They will not receive kernel privileges. The play-creation loop will run on them exactly the way it runs on a small fixture experience: discover, launch, propose, approve, apply, fork.

If a Tavern-shaped runtime ever ships as an official package, a third-party world-simulation package must be able to coexist with it in the same session. If it cannot, the kernel is the one in error, not the third-party package.

## Stance toward radical creation

The point of Yggdrasil is not to ship a better Tavern. The point is to make experiences possible that the platform's authors did not foresee, and to let players who try them fork, inspect, modify, and share what they discover.

The substrate is biased toward this. Events are append-only and kernel-owned ordering. Branches are first-class. Proposals are auditable. Surfaces are descriptors, not hardcoded UIs. Packages are equal regardless of origin or entry form.

When a feature decision makes radical creation harder — by privileging an official path, by hiding state from inspection, by forcing a single shape — that is a charter regression and the feature loses.

## Stance toward "release"

There is no "1.0 chat experience" target. The platform's release shape is:

- Foundation Alpha — the substrate is content-free and credible (current).
- Playable Experience Alpha — at least one experience runs end-to-end on substrate, replaceable, forkable, assistant-assistable.
- Authoring Beta — third parties can ship packages with the same status as official ones.
- Substrate v1 — the substrate stops moving fast enough to commit to public protocol stability.

Anything past Substrate v1 is product range. The platform never owns it.
