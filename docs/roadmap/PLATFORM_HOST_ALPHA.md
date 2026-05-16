# Platform Host Alpha

Platform Host Alpha is the next Yggdrasil milestone after the credible kernel alpha. Its goal is not a content runtime, Studio, Tavern compatibility, pi integration, or a game framework. Its goal is to prove that Yggdrasil can host unprivileged external packages through the same public contract used by every caller.

The first executable slice of this milestone is now in place. The milestone remains open until the remaining partial items below are complete, but the host can already run a third-party subprocess package, call it through the protocol dispatcher, enforce core permissions/schema/timeout behavior, and prove that path through hostile conformance.

## Milestone definition

Platform Host Alpha is complete when a fresh host with zero official packages can:

1. load a third-party subprocess package from its manifest,
2. complete a JSON-RPC-over-stdio lifecycle handshake,
3. expose and invoke a capability through the public protocol,
4. enforce package permissions, namespace ownership, schemas, timeouts, and process teardown,
5. dispatch declared hooks deterministically for the implemented extension points,
6. unload the package cleanly,
7. pass hostile conformance through in-process and public transport paths.

## Implemented slice

1. Protocol and principal foundation: method envelopes, runtime context, structured errors, and no caller-supplied package identity spoofing for package-principal paths.
2. Subprocess package execution: JSON-RPC stdio start, handshake, invoke, invoke timeout, degraded state, and unload kill.
3. Public transports: canonical HTTP `/rpc` and host JSON-RPC stdio mode for non-streaming methods.
4. Hook fabric slice: event append and capability invoke before/after dispatch, stable ordering, veto fixture, metadata mutation fixture, and unload cleanup.
5. Package authoring harness: Python subprocess template, package check, local fixture run, local invoke, and package conformance.
6. Release-gate conformance: named hostile cases with docs matrix coverage.

## Remaining Platform Host Alpha work

1. Sequence-range event replay and resumable subscribe.
2. Real package-owned hook handler execution instead of fixture handler names.
3. Explicit lifecycle transition events for loading/starting/ready/stopping/stopped/degraded.
4. Version-constraint routing and provider selection policy.
5. Broader transport parity cases beyond the current core protocol dispatcher/service tests.
6. Node package template once the subprocess helper boundary is stable enough to duplicate.

## Non-goals for this milestone

- conversational runtime,
- model provider package,
- SillyTavern compatibility,
- pi integration,
- Studio / Prompt Inspector UI,
- game, world, actor, director, memory, or agent semantics,
- marketplace or package dependency resolution,
- remote package execution,
- WASM package execution,
- full OS sandbox guarantees beyond explicitly tested subprocess timeout/kill behavior.

## Required invariant

No official package, client, service route, or SDK helper may use a privileged kernel path. Official namespaces are ordinary namespaces. If a behavior is unavailable to a third-party package through the public protocol and manifest model, it is not a Platform Host Alpha feature.
