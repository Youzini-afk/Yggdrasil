# SillyTavern successor project

> [English](./TAVERN_COMPAT.en.md) · [中文](./TAVERN_COMPAT.md)

The SillyTavern successor is called **YdlTavern**. It lives in its own repository, not inside Yggdrasil.

- Repo: <https://github.com/Youzini-afk/Yggdrasil-Tavern>
- Position: an integration project that runs on top of Yggdrasil and absorbs SillyTavern users, extensions, character cards, world books, presets, and chat history.
- Goal: near-complete coverage of the SillyTavern API and community resources. The frontend can be rewritten, but the UI structure, styling, and operations should stay familiar to ST users.

YdlTavern consumes Yggdrasil through the public protocol. It doesn't read Yggdrasil internals or rely on private APIs — same standing as any other third-party project.

## Why it doesn't live in this repo

Yggdrasil is the platform. Putting a product-grade project — with hundreds of extension shims and six years of API surface to absorb — into `packages/official/` would immediately violate the charter: "official packages have no privileges."

YdlTavern is large, full of product decisions, and tied directly to the SillyTavern community. Those are product concerns, not platform concerns. The two have to stay separate.

## What Yggdrasil keeps providing

These are already in Yggdrasil and YdlTavern will use them directly:

- The public protocol (HTTP `/rpc` plus SSE event subscription).
- `secret_ref`, network declarations, outbound audit, streaming and cancel lifecycle.
- Model integration packages: OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, Fireworks.
- Generic creative capability packages: persona-lab, knowledge-lab, context-lab, text-transform-lab, memory-lab.
- The proposal/approval lifecycle, assets, branches, projections.
- Coming soon: installing capability packages from a GitHub address — YdlTavern's extension ecosystem will benefit.

## What the kernel will never do

No matter how big or important YdlTavern gets:

- The kernel will not understand character cards, world books, presets, or prompt rendering.
- The kernel will not hardcode `{{char}}` / `{{user}}` substitution.
- The kernel will not offer Tavern-specific hooks or methods.
- The kernel will not treat Tavern-shaped packages differently from any other package.

## TavernHeadless research

[`integrations/tavern-headless/`](../../integrations/tavern-headless/) stays in the Yggdrasil repo as research notes. It informs Yggdrasil's generic capability packages (persona / knowledge / context / model-provider). That layer is a platform concern, kept separate from YdlTavern as an integration project.

YdlTavern's actual compatibility roadmap, extension bridge design, and UI structure live in the YdlTavern repo.
