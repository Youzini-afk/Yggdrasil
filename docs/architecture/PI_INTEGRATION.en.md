# pi Integration Boundary

> [English](./PI_INTEGRATION.en.md) · [中文](./PI_INTEGRATION.md)

This document fixes how Yggdrasil absorbs agent-framework capabilities from [pi](https://github.com/earendil-works/pi). pi is an important reference for agent package infrastructure, and it may be used inside packages. It is not the Yggdrasil kernel, protocol, or product shell.

## Current position

Yggdrasil should be able to host, constrain, observe, and replace agent-like capability packages. Yggdrasil itself should not become a built-in agent runtime.

Agent infrastructure must sit on existing public primitives:

- `kernel.v1.capability.invoke` / `kernel.v1.capability.stream` starts or advances an agent-like package capability.
- `kernel.v1.capability.cancel` cancels an in-flight stream invocation.
- `kernel.v1.capability.discover` / `kernel.v1.capability.describe` discovers capabilities that can be adapted into agent tools.
- `kernel.v1.proposal.create/get/list/approve/reject/apply` carries agent-produced change proposals.
- `kernel.v1.event.append/list/subscribe` carries package-owned trace, tool-call, and run events.
- `kernel.v1.surface.contribution.*` lets Assist / Forge discover agent actions and trace panels through public protocol.
- Permissions, `secret_ref`, network declarations, outbound audit/redaction, and stream/cancel lifecycle continue to be enforced by the secure execution substrate.

## pi layer absorption strategy

| pi layer | Yggdrasil handling | Reason |
|---|---|---|
| `pi-ai` | Reference + future ordinary model/inference package internal option | Provider registry, stream/tool-call shape, and faux provider are valuable. Real model calls still require mature host policy, secret, network, audit, usage, and redaction contracts. |
| `pi-agent-core` | Adapter now + package-internal optional | `AgentEvent`, `AgentTool`, before/after tool-call hooks, parallel/sequential execution, and steer/followUp queues are worth absorbing. model/message/systemPrompt/thinkingLevel must not enter the kernel.v1. |
| `pi-coding-agent` | Reference only | It is a complete coding-agent product with TUI, bash/read/write/edit tools, session JSONL, model resolver, skills/extensions, and coding workflow. It is not suitable as a Ygg platform dependency or product shell. |

For the finer ledger, see [`../../integrations/pi/README.md`](../../integrations/pi/README.md).

## Mapping agent concepts to Ygg primitives

| Agent concept | Yggdrasil public primitive | Rule |
|---|---|---|
| run / turn / step | package capability invocation or stream invocation | The kernel does not gain an agent lifecycle. |
| cancellation | `kernel.v1.capability.cancel` | Use the generic stream/cancel lifecycle. |
| tool discovery | `kernel.v1.capability.discover` / `describe` | A tool is an adapter view of a capability. |
| tool execution | `kernel.v1.capability.invoke` / `stream` | Preserve caller principal, provider package, permission gate, and audit. |
| tool ambiguity | explicit `provider_package_id` | Never prefer official providers automatically. |
| proposal | `kernel.v1.proposal.*` | Agents do not directly mutate trusted state. |
| trace | package-owned events or stream frames | The kernel does not interpret trace payloads. |
| state | package-owned asset/projection/get_state capability | No `kernel.v1.agent.state`. |
| memory/prompt/model | future ordinary packages | Not kernel concepts. |
| UI | surface contributions + public protocol | Assist/Forge do not read runtime internals. |

## SDK and package boundaries

Future agent infrastructure may add:

- `sdk/typescript/ygg-agent-adapter`: maps Ygg capabilities to pi-style tools and provides proposal, trace, stream/cancel, and permission/provider diagnostics helpers.
- `ygg init-package --template agent-runtime`: generates a subprocess agent package template with no network by default.
- `official/pi-agent-runtime-lab`: ordinary reference package, no-network/faux by default, no real model calls.
- `official/capability-tool-bridge-lab`: ordinary tool bridge package for discovery, permission preview, explicit provider selection, and public-protocol calls.
- Forge/Assist observability for agent traces, tools, and proposals.
- Third-party replacement proof showing official agent packages have no priority.

These components must not:

- import runtime private modules;
- bypass package/capability/permission/proposal boundaries;
- hardcode official package IDs in UI;
- expose raw secrets in events/proposals/audit;
- provide default bash/edit/write tools;
- make real model calls at this stage.

## Kernel non-goals

The kernel will not add or standardize:

- `kernel.v1.agent.*`
- `kernel.v1.model.*`
- `kernel.v1.prompt.*`
- `kernel.v1.memory.*`
- `kernel.v1.turn.*`
- agent state, chat transcript, prompt template, model provider, thinking/reasoning, or memory taxonomy.

## Anti-patterns

- Embedding `pi-coding-agent` as the Ygg product shell.
- `Assist` starting agents through private runtime paths.
- A tool bridge automatically selecting the first matching provider or preferring official providers.
- Agent packages directly writing asset/projection/session trusted state.
- Storing pi `AgentState` as kernel state.
- Adding a kernel trace ontology for a trace viewer.
- Connecting real OpenAI/Anthropic first and adding secret/network/audit/redaction later.

## Current status

Agent infrastructure has entered execution. This document fixes the boundary and ledger. Next work adds the adapter SDK, a no-network-by-default template, an ordinary official reference package, the tool bridge, Forge/Assist observability, and third-party replacement proof. Real model inference remains deferred until a dedicated package and host policy are ready.
