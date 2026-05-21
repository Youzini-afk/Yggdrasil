# Vision

> [English](./VISION.en.md) · [中文](./VISION.md)

Yggdrasil is an extensible creation platform for AI-native worlds, games, stories, and play.

The center is stable, restrained, and has no opinion about content. Every meaningful concept lives in capability packages, and the kernel hosts them all on equal terms.

## What Yggdrasil is

A kernel that hosts capability packages.

A public protocol that lets clients, packages, and outside systems join on equal footing.

An event log that preserves what happened.

A creation substrate for radical AI-native experiences — without prescribing what those experiences look like or who builds them.

## What Yggdrasil isn't

Not an application, not a chat tool, not a SillyTavern replacement, not a framework with built-in genres. The kernel doesn't carry privileged official content.

The platform won't ship a flagship experience. The kernel takes no stance on characters, worlds, prompts, models, agents, or memory. Those are package concerns.

## What "radical creation freedom" means here

Creators aren't confined to shapes the platform imagined.

A creator can:

- define their own genres, loops, rules, and presentation;
- compose AI behavior like building blocks;
- inspect, fork, rewrite, and recombine any experience;
- replace or override any official package with one of their own;
- ship new capabilities, new event kinds, new extension points;
- mix multiple packages in one session, with no privileged participant.

The platform's job is to make that possible — not to ship the experience itself.

## Why kernel + capability packages

Closed frameworks decide what the medium is. Yggdrasil refuses to do that.

Putting all meaning in packages — including official ones — keeps the medium open. It also keeps the platform honest: if an official "conversation runtime" can be replaced, or can coexist with a third-party "world simulator," then the kernel isn't quietly in charge.

That's how creation freedom is protected over time.

## Where Yggdrasil fits

Yggdrasil is designed to serve as:

- a local platform host;
- a headless service that speaks the public protocol;
- a library embedded in a larger product;
- an open protocol endpoint that outside systems join as packages or clients.

All four use the same contract.

## Deferred capability families

These are valuable directions, but they belong in capability packages, not the kernel. They wait until the kernel and package layers are stable.

- The SillyTavern successor project YdlTavern — runs on top of Yggdrasil as an integration project, absorbing SillyTavern users and community resources. See [`../tavern/TAVERN_COMPAT.md`](../tavern/TAVERN_COMPAT.en.md).
- An agent integration package family (pi or otherwise).
- A game-engine bridge family (UE5, Godot, Unity, web clients).
- An official conversation runtime package.
- An official inspector and creator UI.

Each will be built and judged as an ordinary capability package. None will get kernel privilege.

## Non-goals

The kernel will not ship a chat experience, a world simulator, a director, a memory model, a SillyTavern compatibility layer, an external engine bridge, or an official UI.

Each of those is fine as a capability package; none of them belong in the kernel.

## Stance on the current code

The Rust workspace today is at Platform Foundation Alpha: kernel-only events/sessions, manifest-driven packages, real `rust_inproc` and subprocess execution, the hook fabric, the SQLite event log, principals with permissions, surface contributions, the proposal/approval lifecycle, the asset/branch/projection substrate, and a web shell that speaks the public protocol. The current discipline is preventing contract drift — surfaces, proposals, branches, assets, and projections must stay generic, content shapes don't leak into the kernel, and official packages only use what any third-party package can use.

## What success looks like

Yggdrasil succeeds when a creator builds something the platform's authors never anticipated, ships it as a capability package, and runs it next to official packages without being treated as a second-class citizen.
