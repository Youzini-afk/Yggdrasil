# Yggdrasil web shell

> English only for now. The bilingual project entry point is in the repository root README.

Public-protocol Home/Play, Forge, and Assist shell for the current Platform Foundation Alpha surface.

- `Home/Play` is the launcher-first surface for package-discovered experiences.
- `Forge` is the creation and inspection surface for sessions, events, proposals, capabilities, surfaces, assets, projections, and package labs.
- `Assist` is a drawer that bridges lightweight play edits and deeper Forge work through approval-gated proposals.
- **Text Surface Proof (Phase T1)** is a client-side UI proof inside the Assistant Drawer. It demonstrates progressive mock-streaming text with live line/height estimates using a lightweight text-layout adapter (`src/text-layout`). This is not a kernel feature and does not depend on real model/agent calls.

This client uses only public host APIs:

- `POST /rpc`
- `GET /kernel/event.subscribe/:session_id`

Run the host first:

```bash
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
```

Then serve `clients/web` with any static web server. This is intentionally not a final visual design or a content runtime.

## Text Surface Proof

The `src/text-layout` module provides a Pretext-aligned API shape with a browser-only canvas fallback:

- `prepareText(text, font, options?)` → opaque handle
- `layoutPreparedText(prepared, maxWidth, lineHeight)` → `{ lineCount, height }`
- `createStreamingBuffer(font, lineHeight, maxWidth)` → streaming accumulator with `append`, `end`, `reset`, `measure`, `layoutLines`

It compiles and runs without installing `@chenglou/pretext`. If Pretext is adopted later, swap the internal implementations while keeping the same types and callers.

The proof draws from `src/text-layout/mock.ts` (inert chunks, no network) and renders inside the Assistant Drawer with state badges (`idle`, `streaming`, `ended`, `reset`) and live metadata.
