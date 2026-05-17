# Package Authoring Walkthrough

> [English](./PACKAGE_AUTHORING_WALKTHROUGH.md) · [中文](./PACKAGE_AUTHORING_WALKTHROUGH.zh-CN.md)

This walkthrough creates a third-party package that appears in Home, contributes Forge and assistant surfaces, passes local conformance, and can be composed with other packages. It deliberately uses the same public manifest/capability/surface path as official packages.

## 1. Generate a package

```bash
cargo run -p ygg-cli -- init-package /tmp/ygg-seed-package \
  --id example/seed-package \
  --entry subprocess \
  --language typescript-experience
```

The generated manifest includes:

- an `experience_entry` surface for Home;
- a `play_renderer` surface;
- a `forge_panel` surface;
- an `assistant_action` surface;
- one subprocess JSON-RPC capability that echoes input.

## 2. Validate the package locally

```bash
cargo run -p ygg-cli -- package check /tmp/ygg-seed-package/manifest.yaml
cargo run -p ygg-cli -- package conformance /tmp/ygg-seed-package/manifest.yaml
```

These commands only inspect the manifest and invoke the package through the ordinary capability path. They do not grant private host access.

## 3. Create a composition descriptor

```bash
cargo run -p ygg-cli -- init-composition /tmp/ygg-seed-composition --id example/seed-package
cargo run -p ygg-cli -- composition check /tmp/ygg-seed-composition/composition.yaml
```

A composition descriptor says which packages provide the launchable entry and which surface slots must be present. It is not a kernel `game` or `experience` type.

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

## 5. Compare with official packages

Official packages under `packages/official/` are reference implementations, not privileged routes:

- `official/composition-lab` explains launch plans and surface graphs.
- `official/asset-lab` previews assets and drafts import plans.
- `official/projection-lab` explains projection rebuilds and source events.
- `official/playable-seed` proves a reference playable package.

A third-party package should be able to replace any of these when it exposes compatible surfaces and capabilities.

## Invariants

- Packages must not self-assert caller identity.
- Packages must write only inside authorized namespaces.
- Assistant-like packages must return proposals or events, not mutate trusted state directly.
- UI and tooling must use public protocol methods only.
- If a capability needs mutation, route it through permission checks and `kernel.proposal.*` when user approval is required.
