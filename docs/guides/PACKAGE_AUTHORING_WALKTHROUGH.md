# Package Authoring Walkthrough

> [English](./PACKAGE_AUTHORING_WALKTHROUGH.md) · [中文](./PACKAGE_AUTHORING_WALKTHROUGH.zh-CN.md)

This walkthrough creates a third-party package that appears in Home, contributes Forge and assistant surfaces, passes local conformance, and can be composed with other packages. It deliberately uses the same public manifest/capability/surface path as official packages.

## 1. Generate a package

```bash
cargo run -p ygg-cli -- init-package /tmp/ygg-seed-package \
  --id example/seed-package \
  --entry subprocess \
  --language typescript \
  --template full-surface
```

The generated manifest includes:

- an `experience_entry` surface for Home;
- a `play_renderer` surface;
- a `forge_panel` surface;
- an `assistant_action` surface;
- an `asset_editor` surface;
- one subprocess JSON-RPC capability that echoes input.

For narrower packages, select another template:

```bash
cargo run -p ygg-cli -- init-package /tmp/ygg-assist \
  --id example/assist \
  --entry subprocess \
  --language typescript \
  --template assistant-action

cargo run -p ygg-cli -- init-package /tmp/ygg-asset-editor \
  --id example/asset-editor \
  --entry subprocess \
  --language python \
  --template asset-editor
```

Available templates are:

- `basic` — capability only, no surfaces.
- `experience` — Home `experience_entry` only.
- `play-renderer` — Play renderer surface.
- `forge-panel` — Forge panel surface.
- `assistant-action` — assistant action surface with approval policy metadata.
- `asset-editor` — asset editor surface.
- `full-surface` — all authoring/play surface slots.

`--language typescript-experience` remains supported as a legacy shortcut for a full experience-shaped package.

## 2. Validate the package locally

```bash
cargo run -p ygg-cli -- package check /tmp/ygg-seed-package/manifest.yaml
cargo run -p ygg-cli -- package conformance /tmp/ygg-seed-package/manifest.yaml
cargo run -p ygg-cli -- package run-fixture /tmp/ygg-seed-package/manifest.yaml
cargo run -p ygg-cli -- package reload /tmp/ygg-seed-package/manifest.yaml
```

These commands only inspect the manifest and invoke the package through the ordinary capability path. They do not grant private host access.

`package check` prints authoring diagnostics such as entry kind, trust level, capability count, surfaces by slot, permission summary, sandbox policy, and warnings for packages with no capabilities or no surfaces. `package run-fixture` invokes declared non-streaming capabilities with deterministic fixture input and prints a structured JSON result. `package reload` exercises the local load/restart/unload loop and reports package status and logs.

## 3. Create a composition descriptor

```bash
cargo run -p ygg-cli -- init-composition /tmp/ygg-seed-composition --id example/seed-package
cargo run -p ygg-cli -- composition check /tmp/ygg-seed-composition/composition.yaml
```

A composition descriptor says which packages provide the launchable entry and which surface slots must be present. It is not a kernel `game` or `experience` type.

Composition descriptor v2 fields can also declare optional packages, required capabilities, permission expectations, replacement candidates, default activation metadata, and compatibility notes. `composition check` reports loaded package paths, surfaces by slot, capabilities, missing required surfaces/capabilities, optional-package warnings, and replacement diagnostics.

For a replacement proof, inspect the included third-party example:

```bash
cargo run -p ygg-cli -- package check examples/packages/thirdparty-playable-seed/manifest.yaml
cargo run -p ygg-cli -- composition check examples/compositions/playable-seed-replacement/composition.yaml
```

The package id is `thirdparty/playable-seed`, not `official/*`, and it exposes compatible Play/Forging/Assistant/Asset surfaces without official priority.

## 4. Load the package in a host profile

Add the package manifest to a host profile, for example:

```yaml
autoload:
  - /tmp/ygg-seed-package/manifest.yaml
```

Then run:

```bash
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
```

Home discovers the package through `kernel.surface.contribution.list`. Forge discovers panels through the same protocol. The UI does not receive private runtime handles.

Forge now includes lightweight authoring panels over public protocol data:

- package and capability inventory grouped by provider package;
- surface inventory grouped by slot;
- authoring diagnostics for packages, capabilities, surfaces, assets, projections, and entry surfaces;
- copy-ready CLI command guidance for templates, package checks, fixture runs, reloads, and compositions.

## 5. Compare with official packages

Official packages under `packages/official/` are reference implementations, not privileged routes:

- `official/composition-lab` explains launch plans and surface graphs.
- `official/asset-lab` previews assets and drafts import plans.
- `official/projection-lab` explains projection rebuilds and source events.
- `official/playable-seed` proves a reference playable package.

A third-party package should be able to replace any of these when it exposes compatible surfaces and capabilities.

The `examples/packages/thirdparty-playable-seed` package is the current proof. Conformance verifies that its surfaces are discoverable, capabilities invoke through normal routing, composition checks pass, and shared capability ids are rejected as ambiguous unless an explicit provider is selected. There is no implicit official priority.

## Invariants

- Packages must not self-assert caller identity.
- Packages must write only inside authorized namespaces.
- Assistant-like packages must return proposals or events, not mutate trusted state directly.
- UI and tooling must use public protocol methods only.
- If a capability needs mutation, route it through permission checks and `kernel.proposal.*` when user approval is required.
