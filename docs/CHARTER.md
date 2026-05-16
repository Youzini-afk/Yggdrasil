# Yggdrasil Charter

Yggdrasil is an extension-driven creation platform for AI-native worlds, games, stories, and play.

It is a kernel and a contract — small, stable, opinion-free at the center — over which an open ecosystem of capability packages provides every meaningful concept.

This charter pins down what Yggdrasil is, what it is not, and the principles that do not change.

## Identity

Yggdrasil is:

- a kernel for hosting capability packages,
- a public protocol for clients, packages, and external systems to participate as equals,
- an event-sourced foundation that preserves what happened,
- a creation surface for radically open AI-native experiences.

Yggdrasil is not:

- an application,
- a chat tool,
- a SillyTavern replacement,
- a framework with built-in genres, loops, or content models,
- a plugin host whose center is filled with privileged official content.

## Permanent principles

### 1. The kernel knows nothing about content

No characters, scenes, worlds, prompts, models, turns, chats, agents, memories, games, rules, dice, inventories, or genres are part of the kernel. All such concepts live in capability packages. If a concept is meaningful to a creator or a player, it does not belong in the kernel.

### 2. Official packages have no privileges

Anything an official package can do, a third-party package can do. There are no private APIs, no special hooks, no hidden lifecycle, no kernel shortcuts based on package id or origin.

### 3. Protocol-first

The kernel exposes one set of contracts. Studio, CLI, in-process packages, subprocess packages, WASM packages, and remote services all speak the same protocol. Internal callers do not bypass it.

### 4. Many entry forms, equal status

A capability package may be:

- a Rust crate (in-process),
- a local subprocess speaking JSON-RPC,
- a WASM module,
- a remote HTTP/WebSocket service.

Same manifest, same fabric, same contract. Packaging form is implementation detail.

### 5. Events are the truth

The kernel maintains an append-only event log per session as the system's source of truth. Anything stateful is derived. The kernel does not interpret event payloads; packages do.

### 6. Sandbox by declaration

Side effects, network access, persistence reach, and cross-package calls are declared in manifest. The kernel enforces those declarations. Undeclared side effects are violations.

### 7. Composition over containment

The platform never owns a "main experience." Multiple packages can coexist in a session, layering capabilities, hooks, and presentations. There is no canonical mode.

## Stance toward radical creation

Creators should be able to:

- define their own genres, loops, and rules,
- compose AI behaviors as building blocks,
- inspect, branch, rewrite, and recombine any experience,
- be limited only by what they can express, not by what the platform expected.

The platform's job is to make this possible, not to provide the experience.

## Non-goals

The kernel will not ship:

- a chat experience,
- a world simulator,
- a director or narrator,
- a memory model or retrieval strategy,
- a SillyTavern compatibility layer,
- an external game engine bridge,
- a blessed UI.

Each of these is appropriate as a capability package. None is appropriate as kernel.

## Stance toward today's code

The current Rust workspace contains conversational concepts (`Turn`, `PromptFrame`, `ModelCall`, message commit) inside what should be a content-free kernel. This is a known deviation. The first refactor lands the kernel/package separation. Until then, today's code is treated as a working spike, not as the final shape.

## Stability commitment

This charter changes only by explicit revision. The kernel may evolve; the principles do not. When a future feature appears to require violating a principle, the answer is to redesign the feature, not the principle.
