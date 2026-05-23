# Round 9 (C-track) — Contract Foundation Plan

> Temporary planning document. Removed at C11 once docs converge.

## Why this round

Yggdrasil's central thesis is **"kernel + contract"**: a kernel implementation
plus a public contract that any participant can use to invoke the kernel —
across any language, in any of four entry forms (rust_inproc / subprocess /
wasm / remote), AND opt out entirely (path B: self-contained apps that don't
use the kernel at all).

Today the contract is **implicit**. Schema lives in Rust source. Method names
are stringly typed. Capabilities are coarse permission lists. There is no
public IDL artifact, no conformance kit, no explicit Path B. Third parties
who want to write a package would have to read Rust source.

Round 9 makes the contract **explicit, public, and machine-checkable** —
without locking ourselves into one IDL.

## Constraints (user feedback)

- **No deprecation layer**: kernel hasn't shipped, so we rename freely. No
  `kernel.v1.session.open` AND `kernel.v1.session.open` coexisting. Clean break.
- **npm is one distribution path, not the only one**: SDK consumers can clone
  the repo and use a path reference, install the published npm package, or
  read the public schemas and generate their own bindings.
- **Path B is first-class**: apps that don't use the kernel are equally valid
  participants on the platform.

## Out of scope (defer to Round 10)

- WIT worlds + WASM entry form (parallel track, not blocking)
- Powerbox late-bound provider selection (advanced authority pattern)
- Cap'n Proto Level 4 RPC for remote
- Biscuit attenuation tokens

## Phases

### C1 — Schema extraction + v1 namespace (clean break)

Extract all kernel.v1.* method shapes and event kinds from Rust source into
public JSON Schema 2020-12 files. Rename every method to `kernel.v1.*`.

Outputs:
- `docs/spec/v1/schemas/manifest.schema.json`
- `docs/spec/v1/schemas/methods/<method>.schema.json` (one per method)
- `docs/spec/v1/schemas/events/<event-kind>.schema.json`
- `docs/spec/v1/EVENT_KIND_REGISTRY.md`
- `docs/spec/v1/ERROR_CODES.md`
- `docs/spec/v1/VERSIONING.md`
- All references in protocol.rs, protocol_dispatch.rs, conformance,
  subprocess SDK, HTTP service, fixtures, examples renamed to v1.

No backward-compat shims.

### C2 — Capability handle table

Introduce kernel-minted `CapHandle` as the runtime representation of granted
authority. Manifest strings become *ceilings*; handles become *actual
authority*.

New types:
- `CapHandleId` (opaque u128, kernel-minted)
- `CapHandle { id, cap_type, cap_version, scope, constraints, lease,
  provenance, parent? }`
- `HandleTable` (per-package, indexed by HandleId, supports mint/lookup/
  revoke/list)

New methods:
- `kernel.v1.cap.attenuate(handle, constraints) -> handle'`
- `kernel.v1.cap.revoke(handle)`
- `kernel.v1.cap.list_for(package_id)`

Modified methods:
- `kernel.v1.capability.invoke` accepts either `handle: HandleId` (new
  preferred path) OR `capability_id: String` (auto-mints transient handle for
  packages that haven't migrated yet).

### C3 — Bindings injection (entry-form-specific)

Each entry form receives bindings from the kernel at startup:

- **subprocess**: handshake response carries `bindings: { <cap_logical_name>:
  HandleId }`. SDK exposes them as typed methods on a `kernel` object.
- **rust_inproc**: `KernelEnv` parameter passed at registration; provides
  typed `CapHandle<T>` wrappers.
- **wasm** (scaffolded only): planned WIT resource imports — drop a stub
  documenting the design.
- **remote** (scaffolded only): planned SPIFFE + Biscuit token exchange —
  drop a stub.

### C5 — Effect audit (declared vs used)

Per-package, kernel records actual capability use over time. CLI command
`yg audit --package <id>` reports declared maximum vs used in last N
sessions, suggests least-authority manifests.

### C6 — Capability invoke instrumentation

Add `duration_ms` to `CapabilityInvocationResult`. Write the three event
kinds that the spec already declared but no one writes:

- `kernel/v1/v1/capability.invoked`
- `kernel/v1/v1/capability.completed`
- `kernel/v1/v1/capability.failed`

Add `correlation_id` and `parent_invocation_id` fields to `ProtocolContext`
for trace correlation. Outbound completion events (already have duration_ms
since Z6) cross-link via correlation_id.

### C7 — Path B manifest field

Add `entry.contract` field to manifest:
- `"v1"` (default): full contract enforcement
- `"none"`: opt-out — kernel hosts the process but doesn't enforce
  capability/permission checks

`contract: "none"` packages can use the kernel through reverse RPC if they
want (still optional), but they're not bound to declare permissions or
capabilities. This is the "self-contained app" pattern.

### C9 — Conformance test kit

`yg conformance --contract v1 --package <path>` runs:

1. Manifest schema validation
2. Handshake feature negotiation
3. Each declared capability gets a smoke invocation
4. Streaming capabilities exercise cancel/timeout
5. Permission denial paths emit correct audit
6. Handle lifecycle (mint/attenuate/revoke)
7. Error codes match `ERROR_CODES.md`
8. Event kinds match `EVENT_KIND_REGISTRY.md`

Reports per-package conformance %.

### C10 — SDK generation (multi-distribution)

Add a build script that generates SDKs from `docs/spec/v1/schemas/`:

- `sdk/typescript/` — generated TS types + JSON-RPC client. Distribution
  options:
  - npm publish (`@yggdrasil/kernel-sdk` package)
  - workspace path (clone repo, reference `sdk/typescript/`)
  - read schemas, generate your own
- `sdk/rust/` — generated Rust types + traits.
- `sdk/openapi.yaml` — generated for HTTP/REST consumers (non-canonical, for
  arbitrary OpenAPI generators).

Existing handwritten subprocess SDK is replaced by generated code at the
same path, with the same surface (`kernelClient.invokeCapability(...)` etc.)
to keep YdlTavern compiling.

### C11 — Docs convergence + delete plan + final validation

- Replace `docs/spec/KERNEL_V0_ALPHA_CONTRACT.md` with
  `docs/spec/KERNEL_V1_CONTRACT.md` (no v0 vestiges).
- New `docs/guides/PATH_B_SELF_CONTAINED.md` (bilingual).
- New `docs/guides/CONFORMANCE_KIT.md` (bilingual).
- New `docs/guides/CAPABILITY_HANDLES.md` (bilingual).
- Update `README` / `ARCHITECTURE` / `ALPHA_STATUS` / `NEXT_STEPS`.
- Update `CONFORMANCE_MATRIX` to reflect C9 coverage.
- Update YdlTavern docs that reference the contract (manifest, kernel
  methods).
- Delete `docs/CONTRACT_FOUNDATION_PLAN.md` (this file).
- Run final cross-repo validation (test counts + golden harness).

## Wave plan

```
Wave 1 (parallel): C1, C7, C6
Wave 2 (sequential): C2
Wave 3 (parallel): C3, C5
Wave 4 (parallel): C9, C10
Wave 5 (sequential): C11
```

After each wave, commit + push to both repos as needed.

## Push cadence

Each phase push is one commit per repo. Compound commits at wave boundaries
when a wave produces work in multiple phases.

## Cross-repo touchpoints

- C1 rename touches YdlTavern's hardcoded method strings if any. After C1,
  YdlTavern's `ydltavern-engine` capability registrations need their kernel
  reverse calls updated to `kernel.v1.*`.
- C3 bindings change YdlTavern's subprocess startup handshake handling. The
  generated SDK from C10 should keep the same TS surface, so YdlTavern's
  call sites don't change much.
- C9 conformance kit is run against YdlTavern's manifest at the end of C11.
- C11 docs update YdlTavern's references to `kernel.v1.v0.alpha` to `kernel.v1.v1`.

YdlTavern won't be massively rewritten; it's mostly method-name updates and
manifest field additions.
