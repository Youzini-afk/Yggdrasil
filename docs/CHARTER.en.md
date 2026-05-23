# Yggdrasil Charter

> [English](./CHARTER.en.md) · [中文](./CHARTER.md)

Yggdrasil is an extensible creation platform for AI-native worlds, games, stories, and play.

It has two parts: a small, restrained, opinion-free kernel, and an open ecosystem of capability packages. Every meaningful concept on the platform comes from a package.

This charter defines what Yggdrasil is, what it isn't, and the principles that don't change.

## Identity

Yggdrasil is:

- a kernel that hosts capability packages;
- a public protocol that lets clients, packages, and outside systems join on equal footing;
- an event log that preserves what happened;
- a creation substrate for radical, open AI-native experiences.

Yggdrasil is not:

- an application;
- a chat tool;
- a SillyTavern replacement;
- a framework with built-in genres, loops, or content models;
- a plugin host whose kernel is full of privileged official content.

## Principles that don't change

### 1. The kernel knows nothing about content

Characters, scenes, worlds, prompts, models, turns, chats, agents, memory, games, rules, dice, inventories, genres — none of these live in the kernel.v1. They are package concerns. If a concept means something to a player or a creator, it doesn't belong in the kernel.v1.

### 2. Official packages have no privileges

Anything an official package can do, a third-party package can do too. No private APIs, no special hooks, no hidden lifecycles, no kernel shortcuts based on package id.

### 3. Protocol first

The kernel exposes one contract. Studio, the CLI, in-process packages, subprocess packages, WASM packages, and remote services all use it. Internal callers don't get to bypass it either.

### 4. Entry forms are equal

A capability package can be:

- a Rust crate (in-process),
- a local subprocess speaking JSON-RPC,
- a WASM module,
- a remote HTTP/WebSocket service.

All four share the same manifest format, the same fabric, the same contract. Packaging form is an implementation detail.

### 5. Events are truth

Each session has an append-only event log as the source of truth. Anything stateful is derived from events. The kernel doesn't interpret event payloads — packages do.

### 6. Sandboxing is declarative

Side effects, network access, persistence scope, cross-package calls — all declared in the manifest. The kernel enforces what was declared. An undeclared side effect is a violation.

### 7. Composition over containment

The platform never owns a "main experience." Multiple packages can coexist in one session, layering capabilities, hooks, and presentation. There is no canonical shape.

## Stance on radical creation

Creators should be able to:

- define their own genres, loops, and rules;
- compose AI behavior like building blocks;
- inspect, fork, rewrite, and recombine any experience;
- be limited by what they can express, not by what the platform expected.

The platform's job is to make that possible — not to ship the experience itself.

## Non-goals

The kernel will not ship:

- a chat experience;
- a world simulator;
- a director or narrator;
- a memory model or retrieval strategy;
- a SillyTavern compatibility layer;
- a bridge to an external game engine;
- a privileged UI.

Each of these is welcome as a capability package. None of them belong in the kernel.v1.

## Stance on existing code

The Rust workspace has already removed its early conversation prototype. From here on, putting any content-shaped concept back into the kernel crate counts as a regression against this charter. Conversation, models, memory, agents, worlds, Tavern, Studio — all must arrive as packages.

## Stability promise

This charter changes only by explicit revision. The kernel can evolve; the principles don't. When a future feature seems to require breaking a principle, we redesign the feature, not the principle.
