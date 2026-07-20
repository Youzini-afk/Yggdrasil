# Contract v2 Layering Migration Plan

> [English](./CONTRACT_V2_MIGRATION.en.md) · [中文](./CONTRACT_V2_MIGRATION.md)

> This is a temporary implementation plan. Delete it after the layering migration
> is complete and move durable results into architecture, spec, guides, and status documents.

## Goal

Without discarding the existing runtime, Web client, CLI, SDKs, or conformance, migrate the current `kernel.v1.*` monolithic contract into contracts with explicit ownership, negotiation, and deprecation:

- Constitutional Substrate;
- Host Control Plane;
- Protocol Commons;
- Shell / Product Profiles;
- Legacy Adapters.

Target boundaries are defined in [`CONSTITUTION_V2.md`](../architecture/CONSTITUTION_V2.en.md), with item-level ownership in
[`CONTRACT_LAYERING_MATRIX.md`](../spec/CONTRACT_LAYERING_MATRIX.en.md).

## Non-goals

- Do not rewrite the runtime all at once.
- Do not immediately delete any `kernel.v1.*` method or schema.
- Do not hard-code `World`, `Agent`, or `Surface` back into the substrate.
- Do not immediately require every component to migrate to WASM.
- Do not build a marketplace, remote registry, or economy in this plan.
- Do not treat design documents as evidence that v2 is implemented.

## Migration constraints

1. The current Web client and CLI continue working unchanged during the compatibility window.
2. Every migration adds the canonical route and adapter first, migrates callers second, and deprecates the old name last.
3. Original v1 requests, responses, events, and unknown fields can be preserved losslessly.
4. New contracts default to Experimental and cannot claim Stable before maturity gates pass.
5. Namespace migration never weakens security or authority semantics.
6. Historical replay never triggers network, model, process, or other external effects.
7. Every structural change updates schemas, SDKs, and conformance together.

## Current implementation status

- [x] Phase 1: repair v1 factual drift and the SDK/CI/Windows baselines.
- [x] Phase 2: implement the Experimental Contract Registry, centralized aliases, explicit profile/version negotiation, and the generated SDK/conformance chain.
- [x] Phase 3: add owner-namespace dual stacks for the Host Control Plane, host bundle resolver, Shell contributions, Change/Proposal, and Projection; all 36 legacy aliases still reach the original handlers.
- [ ] Phases 4–9: object/artifact foundations, receipt/change primitives, Protocol Commons, component identity, World Bundle, and client/deprecation migration.

## Immediate freeze line

Until the layering migration is complete:

- add no `kernel.v1.*` methods or events except security fixes, correctness fixes, and compatibility fields;
- put new experiments in an Experimental namespace with an explicit owner rather than expanding v1 stability;
- do not use `project`, `target`, `exec`, `port`, `proxy`, or fixed surface slots as precedent for further substrate expansion;
- incubate new domain semantics in package-owned experiments and admit them to the Protocol Commons only after protocol review.

## Implementation order

### 1. Repair current factual drift

This is low-risk prerequisite work and does not change contract semantics.

Deliverables:

- bring `EVENT_KIND_REGISTRY.md` to 59 events by adding `deployment.health`;
- align `KernelMethod::status()`, the Contract method matrix, and actual dispatch;
- correct Contract capability/outbound namespace counts and the omitted deployment-hub count;
- fix the Rust SDK crate root so generated methods/events/types are exported;
- add all 13 Web tests to CI;
- fix the two runtime tests that hard-code `/tmp` and `python3` on Windows.

Acceptance:

```text
cargo check --workspace
cargo test -p ygg-core
cargo test -p ygg-runtime --lib
cargo test -p ygg-cli
npm run check --prefix clients/web
npm test --prefix clients/web
cargo run -p ygg-cli --bin validate-schemas
```

### 2. Establish the contract registry and alias foundation

Add explicit metadata to the existing `KernelMethod` source of truth without moving handlers first.

Non-normative registry shape:

```text
ContractMethod
├── canonical_id
├── aliases[]
├── owner_layer
├── maturity
├── request_schema
├── response_schema
├── request_adapter
├── response_adapter
├── introduced_in
├── deprecated_in
└── replacement
```

Deliverables:

- dispatcher resolves aliases before invoking the canonical handler;
- aliases live in one registry, never scattered string special cases;
- `host.info` returns layers, versions, profiles, maturity, and aliases;
- generated SDKs expose both canonical clients and legacy aliases;
- conformance adds alias equivalence, unsupported-version rejection, and no-silent-downgrade tests.

The first alias may use identity adapters to prove routing without changing payloads.

Acceptance:

- legacy and canonical IDs return semantically equivalent results;
- permission, principal, audit, and error mapping are identical;
- requesting an unavailable profile fails explicitly instead of falling back to weaker semantics.

### 3. Change ownership without changing behavior

Create dual-stack entries for target namespaces while handlers continue calling current implementation.

First migrations:

- `target/exec/port/proxy/project` → Host Control Plane;
- `surface.*` → host bundle resolver + `ygg.shell.default/v1` profile;
- `proposal.*` → Experimental Change protocol;
- `projection.*` → Experimental Projection protocol.

Old `kernel.v1.*` names remain adapters. This step adds no World, receipt, or object-store data model.

Acceptance:

- the Web client continues working with only the old SDK;
- a new CLI smoke path can use layered namespaces exclusively;
- legacy and canonical routes share handlers and cannot drift into duplicate implementations.

### 4. Establish content-addressed object/artifact foundations

Current `AssetRecord.hash` uses `fnv1a64:`. It is deterministic for tests but unsuitable as collision-resistant, cross-host persistent identity. Current `asset.put` also places full content in event metadata. v2 object identity uses a collision-resistant digest, and journals reference objects.

Deliverables:

- `ArtifactDescriptor { artifact_type_uri, media_type, digest, size_bytes, references, annotations }`;
- `ObjectStore` trait: put, get, has, verify, stream;
- required initial digest `sha256:`, with algorithm prefixes retained for future extension;
- object bytes separated from metadata;
- journal/event/receipt stores descriptors or references, not duplicated large content;
- `asset.put/get/list` adapters map to object/artifact APIs;
- old FNV addresses remain legacy aliases and cannot become v2 canonical identity.

Data migration:

- calculate a SHA-256 descriptor when reading an old asset;
- retain old asset ID, FNV hash, and original v1 event reference;
- migration is idempotent and resumable;
- unknown artifact types can be copied and exported losslessly.

Acceptance:

- identical bytes receive identical digests on two hosts;
- modified bytes fail verification;
- objects can be copied and inspected without loading the originating package;
- large objects no longer appear in full inside event metadata.

### 5. Introduce EffectReceipt and change primitives

Start with effects whose boundaries already exist: capability invocation, outbound HTTP/stream/WebSocket, and local exec.

Deliverables:

- versioned `EffectReceipt` artifact/profile;
- Experimental schemas for `Intent`, `ChangeSet`, `PolicyDecision`, and `Commit`;
- capability/outbound/exec terminal paths produce receipts;
- receipts reference input/output objects, component digest, authority, policy, approval, cost, latency, trace, and parent receipts;
- Proposal adapter maps the old lifecycle to the Change protocol;
- receipts exclude raw bodies, headers, secrets, full prompts, and full user content by default.

Historical replay:

- reads recorded output references without invoking an executor;
- reports incomplete history explicitly when objects are missing;
- re-execution creates a new branch/head and receipt and never overwrites the old receipt.

Acceptance:

- historical calls replay while every outbound executor is disabled;
- re-execution creates a different branch and leaves old history unchanged;
- denied/cancelled/timeout/partial/success all have distinguishable terminal receipts;
- raw-secret scanning covers receipt and adapter output.

### 6. Establish Protocol Commons scaffolding

Deliver a protocol descriptor rather than inventing many domain protocols first.

```text
ProtocolDescriptor
├── protocol_id
├── version
├── maturity
├── schemas / WIT worlds
├── semantic specification
├── lifecycle / state machine
├── error model
├── authority requirements
├── conformance vectors
├── compatibility profiles
├── migrations / adapters
└── conforming implementations
```

Incubate only three initial protocols:

- Change protocol, absorbing the current Proposal;
- Shell Default profile, absorbing fixed `SurfaceSlot` values;
- World Bundle Experimental profile, proving real portability.

Projection remains Experimental until at least two different experiences prove that its shared semantics are stable enough.

Acceptance:

- protocol conformance and package/implementation conformance are reported separately;
- official and third-party implementations use identical vectors;
- a protocol major mismatch has an explicit adapter or rejection and does not rely on schemas merely parsing.

### 7. Separate package envelope from component identity

Deliverables:

- package continues handling retrieval, integrity, and installation transactions;
- artifacts/components/protocols/content/surfaces inside a package each have descriptors and digests;
- component identity is no longer equal to package ID;
- composition lock pins protocol profiles, component digests, and content roots;
- trust class appears in component records and conformance reports;
- `contract:none` maps to `foreign_capsule` and receives no conforming/portable claims.

Execution boundaries:

- `trusted_native` requires explicit host trust;
- `isolated_process` does not claim OS network/filesystem isolation unless the host proves enforcement;
- `sandboxed_component` is the preferred candidate for AI-generated components, but is not mandatory before WASI 0.3 toolchains and host support mature;
- remote implementations make identity, tenancy, network, and revocation semantics explicit.

Acceptance:

- one protocol implementation can ship in different packages while preserving component identity/behavior claims;
- replacing a component does not change content roots;
- Foreign Capsule can start, while conformance clearly reports that composability and portability are not guaranteed.

### 8. Prove the boundary with a real World Bundle

Use `official/playable-creation-board` as the first pressure source instead of building a new large experience.

The Experimental World Bundle contains at least:

```text
WorldBundle
├── bundle_descriptor
├── world_head
├── journal_ranges
├── object_descriptors
├── composition_lock
├── protocol_profiles
├── policy_refs
├── effect_receipts
├── lineage
└── original_v1_envelopes
```

Acceptance flow:

1. Start the playable board on host A and produce state, a branch, and at least one controlled effect.
2. Export the bundle and verify every digest and the full reference closure.
3. Import into host B with a fresh data directory.
4. Without loading the original component or enabling network/model access, audit and replay history deterministically.
5. Install a different implementation, re-execute one step, and create a new branch/head.
6. Read the same world through a headless CLI to prove the Web shell is not a data dependency.

Failure conditions:

- bundle depends on an absolute path from host A;
- history becomes unreadable without the original package;
- replay triggers a real external call;
- unknown artifacts are discarded;
- import changes object digests, lineage, or receipts.

### 9. Migrate clients and begin deprecation

Migration order:

1. generated SDKs;
2. CLI;
3. Web protocol client;
4. subprocess SDK;
5. official packages;
6. third-party examples and guides.

A legacy method enters Deprecated only when:

- its canonical replacement is Candidate or Stable;
- legacy/canonical equivalence conformance passes;
- an SDK and migration tool are published;
- `host.info` reports the replacement;
- the support window is documented;
- at least one real project has migrated.

After entering Legacy Adapter, an old method accepts only security fixes and data-reading compatibility, not new field semantics.

## Conformance reorganization

Retain the existing named-case/tag runner and add these suites:

| Suite | Proves |
|---|---|
| `substrate` | Identity, authority, objects, journals, receipts, streams, transactions |
| `host` | Host behavior for project, exec, port, proxy, secret, deployment |
| `protocol:<id>` | Shared semantics, state machines, errors, behavioral vectors |
| `shell:<profile>` | Descriptor mapping, bridge authority, shell independence |
| `legacy` | Alias equivalence, lossless conversion, support windows, deprecated diagnostics |
| `portability` | Cross-host bundle, offline replay, preservation of unknown artifacts |

Stable substrate release gates include the portability suite, not only single-host unit tests.

## Definition of migration complete

Completion is not “every `kernel.v1.*` method was renamed.” It requires all of the following:

- owner layering agrees across code, schemas, SDKs, and documentation;
- legacy aliases use a central registry and conformance;
- package is no longer the only artifact/component/content identity;
- object identity uses verifiable collision-resistant digests;
- effect receipts cover nondeterminism and external effects;
- Proposal and fixed SurfaceSlot are protocols/profiles;
- Foreign Capsule no longer claims full ecosystem equality;
- World Bundle passes cross-host, offline-replay, and shell-independence tests;
- at least one wrong abstraction completes a real Deprecated → Legacy Adapter migration.
