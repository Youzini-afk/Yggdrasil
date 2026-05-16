# Yggdrasil Vision

Yggdrasil is an extension-driven creation platform for AI-native worlds, games, stories, and play.

The center of the platform is small and content-free. Everything meaningful lives in capability packages that the kernel hosts as equals.

## What Yggdrasil is

A kernel that hosts capability packages.

A public protocol that lets clients, packages, and external systems participate as equals.

An event-sourced foundation that preserves what happened.

A creation surface for radical AI-native experiences that the platform itself does not predefine.

## What Yggdrasil is not

Not an application. Not a chat tool. Not a SillyTavern replacement. Not a framework with built-in genres. Not a plugin host whose center is filled with privileged official content.

There is no canonical experience the platform ships. The kernel has no opinion about characters, worlds, prompts, models, agents, or memory. Those are package concerns.

## What "radical creative freedom" means here

Creators are not limited to the shapes Yggdrasil imagined.

A creator can:

- define their own genres, loops, rules, and presentation,
- compose AI behaviors as building blocks,
- inspect, branch, rewrite, and recombine any experience,
- replace or override any official package with their own,
- distribute new capabilities, new event kinds, new extension points,
- mix multiple packages in one session without one being privileged.

The platform's job is to make this possible. The platform's job is not to provide the experience.

## Why kernel-and-packages

A walled framework decides what the medium is. Yggdrasil refuses to.

Putting all meaning in packages — including the official ones — keeps the medium open. It keeps the platform honest: if an official "conversational runtime" can be replaced or coexist with a third-party "world simulator," then the kernel is not secretly the boss.

This is what protects creative freedom over time.

## Reach

Yggdrasil is designed to be useful as:

- a local platform host,
- a headless service speaking the public protocol,
- an embedded library inside larger products,
- an open protocol for external systems acting as packages or clients.

All four use the same contract.

## Future capability families (deferred)

These are valuable directions, but they are packages, not kernel concerns. They wait until the kernel/package layer is stable.

- A SillyTavern resource and behavior compatibility package family.
- An agent integration package family (pi or otherwise).
- A game engine bridge package family (UE5, Godot, Unity, web clients).
- An official conversational runtime package.
- An official inspector and creator UI.

Each of these will be built and judged as a normal capability package. None will receive kernel privileges.

## Non-goals

The kernel will not ship a chat experience, a world simulator, a director, a memory model, a SillyTavern compatibility layer, an external engine bridge, or a blessed UI.

Each of these is appropriate as a capability package. None is appropriate as kernel.

## Stance toward today's code

The current Rust workspace contains conversational concepts (`Turn`, `PromptFrame`, `ModelCall`, message commit) inside what should be a content-free kernel. This is a known deviation. The first refactor lands the kernel/package separation and migrates today's conversational shape into an official capability package.

## What success looks like

Yggdrasil succeeds when a creator can build something the platform's authors did not foresee, ship it as a package, and have it run alongside the official packages with no second-class treatment.
