# v1 Versioning Policy

## Method namespace

All v1 kernel methods use `kernel.v1.*`. `kernel.v1.*` and future `kernel.v2.*` are separate, incompatible namespaces. v1 method semantics are never changed in breaking ways, and the old `kernel.*` names have no compatibility shim.

## Schema rule

`docs/spec/v1/schemas/` is the public v1 contract artifact. v1 evolves additive-only:

- Optional fields, event kinds, methods, or enum variants may be added when old implementations can safely ignore them.
- Fields must not be removed, field types must not change, optional fields must not become required, published enums must not be narrowed, and existing event payload meaning must not change.
- Schema changes must pass `scripts/validate-schemas.sh`; CI sets `BASE_SCHEMA_DIR` and checks removals plus common structural breakage such as type/const changes, new required fields, enum narrowing, removed properties/definitions, tighter bounds, and incompatible combinator changes.

This additive guarantee applies to the serialized v1 wire contract. The Rust crates and generated SDKs are still pre-1.0: adding an optional wire field can add a field to a public Rust struct and therefore break downstream struct literals. Consumers should prefer constructors, builders, or deserialization and follow each crate or SDK's semantic version when upgrading.

## When v2 happens

Use `kernel.v2.*` for breaking changes: required-field changes, existing field type changes, error-code semantic changes, incompatible permission model changes, or incompatible event payload reshaping. v2 does not overwrite v1; hosts may expose multiple versions side by side.

## Negotiation

Packages call `kernel.v1.host.info` to read `protocol_version`, methods, statuses, and transports. A package needing a v1 method should verify that the method exists and is not `planned`. Future v2 packages should call `kernel.v2.host.info` or inspect the host's explicitly advertised v2 method list.
