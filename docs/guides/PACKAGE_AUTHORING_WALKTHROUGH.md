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
- `networked` — networked capability with declared network permissions (`host`, `methods`, `purpose`), `secret_ref` usage, and outbound audit helper. No raw secrets, no implicit network access. Demonstrates `NetworkDeclaration` and `OutboundAuditHelper` from `sdk/typescript/secure-execution`.
- `streaming` — streaming capability with faux frame lifecycle (`StreamFrameClient`). Demonstrates `start`/`chunk`/`end` frames and `redaction_state`. No real model inference. Uses `sdk/typescript/secure-execution`.

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

## 6. Secure execution helpers

The `sdk/typescript/secure-execution` module provides thin, protocol-safe helpers for packages that need secret references, network declarations, outbound audit, and streaming frames. No private kernel internals are exposed.

### Secret references

```ts
import { secretRef, isValidSecretRef, looksLikeRawSecret } from "../../sdk/typescript/secure-execution/index.js";

// Create a secret reference (never embed raw secrets in payloads)
const ref = secretRef("env", "MY_API_KEY"); // → "secret_ref:env:MY_API_KEY"

// Validate
isValidSecretRef("secret_ref:env:KEY"); // true
isValidSecretRef("sk-abc123");           // false
```

### Network declarations

```ts
import { NetworkDeclaration } from "../../sdk/typescript/secure-execution/index.js";

const decl = new NetworkDeclaration({
  host: "api.example.com",
  methods: ["GET", "POST"],
  purpose: "model inference",
});
decl.toManifestEntry(); // manifest-compatible object
decl.matches("api.example.com", "POST"); // true
```

### Outbound audit helper

```ts
import { OutboundAuditHelper, secretRef } from "../../sdk/typescript/secure-execution/index.js";

const audit = new OutboundAuditHelper({
  packageId: "example/my-package",
  capabilityId: "example/my-package/fetch",
});
const payload = audit.buildRequestPayload({
  destinationHost: "api.example.com",
  method: "POST",
  secretRefsUsed: [secretRef("env", "MY_KEY")],
  purpose: "model inference",
});
// payload contains only references — never raw secrets
```

### Stream frame client

```ts
import { StreamFrameClient } from "../../sdk/typescript/secure-execution/index.js";

const client = new StreamFrameClient();
const startFrame = client.start("example/stream/echo", {});
const chunk1 = client.chunk({ text: "faux token 1" });
const endFrame = client.end();
// Frames carry invocation_id, stream_id, sequence, redaction_state
```

## 7. No-network readiness proof

For packages that want to prove their readiness to work with the secure execution substrate (secret refs, network permissions, streaming) without making real network calls or performing model inference, see the included examples:

```bash
cargo run -p ygg-cli -- package check examples/packages/faux-model-readiness/manifest.yaml
cargo run -p ygg-cli -- package check examples/packages/faux-agent-readiness/manifest.yaml
```

- `example/faux-model-readiness` declares network permissions, uses `secret_ref` for credentials, returns discovery plans (not real API responses), and produces faux streaming frames. No real inference or network calls.
- `example/faux-agent-readiness` produces proposals/traces/plans only, emphasizes public protocol/capability/proposal patterns, has no network permissions, and produces faux streaming trace frames. No connection to pi runtime or model inference.

These packages prove the substrate shape without coupling to any specific model or agent implementation.
