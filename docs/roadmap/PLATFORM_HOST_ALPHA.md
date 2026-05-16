# Platform Host Alpha

Platform Host Alpha is the next Yggdrasil milestone after the credible kernel alpha. Its goal is not a content runtime, Studio, Tavern compatibility, pi integration, or a game framework. Its goal is to prove that Yggdrasil can host unprivileged external packages through the same public contract used by every caller.

## Milestone definition

Platform Host Alpha is complete when a fresh host with zero official packages can:

1. load a third-party subprocess package from its manifest,
2. complete a JSON-RPC-over-stdio lifecycle handshake,
3. expose and invoke a capability through the public protocol,
4. enforce package permissions, namespace ownership, schemas, timeouts, and process teardown,
5. dispatch declared hooks deterministically for the implemented extension points,
6. unload the package cleanly,
7. pass hostile conformance through in-process and public transport paths.

## Current priorities

1. Protocol and principal foundation: method envelopes, runtime context, structured errors, and no transport permission bypass.
2. Subprocess package execution: JSON-RPC stdio start, handshake, invoke, timeout, degraded state, and unload kill.
3. Public transports: canonical HTTP `/rpc`, host JSON-RPC stdio mode, and event replay before live subscribe.
4. Hook fabric completion: event, capability, and package lifecycle hooks with ordering, veto, timeout, and unload cleanup.
5. Package authoring harness: thin Python subprocess template and local package conformance first; Node can follow once the subprocess protocol settles further.
6. Release-gate conformance: hostile cases define the milestone; documentation status must match executable coverage.

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
