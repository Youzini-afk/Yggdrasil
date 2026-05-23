# Yggdrasil Kernel SDK

Generated SDKs for the public kernel contract live in this directory. The source
of truth is `docs/spec/v1/schemas/`; run `scripts/regen-sdks.sh` after changing
Rust contract types or schema exports.

Three distribution channels — pick what fits your workflow:

## Channel 1: npm (TypeScript only)

```bash
npm install @yggdrasil/kernel-sdk
```

Publishing is opt-in. The package is also usable directly from this repository.

## Channel 2: workspace path reference

```bash
git clone yggdrasil
```

Then in your `package.json`:

```json
{ "dependencies": { "@yggdrasil/kernel-sdk": "file:../yggdrasil/sdk/typescript/kernel-sdk" } }
```

Rust consumers can depend on the generated crate by path:

```toml
yg-kernel-sdk = { path = "../yggdrasil/sdk/rust/yg-kernel-sdk" }
```

## Channel 3: read schemas, generate yourself

```bash
git clone yggdrasil
# Use docs/spec/v1/schemas/ with your favorite codegen tool:
quicktype --src-lang schema --lang go docs/spec/v1/schemas/methods/*.json
# or oapi-codegen for Go
# or openapi-generator for any of 50+ languages
# from sdk/openapi.yaml
```

Third-party integrators do not need to consume these SDKs; the JSON Schemas and
OpenAPI description are stable inputs for independent code generation.
