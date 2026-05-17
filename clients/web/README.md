# Yggdrasil web shell

Public-protocol Home/Play, Forge, and Assist shell for the current Platform Foundation Alpha surface.

- `Home/Play` is the launcher-first surface for package-discovered experiences.
- `Forge` is the creation and inspection surface for sessions, events, proposals, capabilities, surfaces, assets, projections, and package labs.
- `Assist` is a drawer that bridges lightweight play edits and deeper Forge work through approval-gated proposals.

This client uses only public host APIs:

- `POST /rpc`
- `GET /kernel/event.subscribe/:session_id`

Run the host first:

```bash
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
```

Then serve `clients/web` with any static web server. This is intentionally not a final visual design or a content runtime.
