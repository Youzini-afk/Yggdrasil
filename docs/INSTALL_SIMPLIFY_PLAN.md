# Round 10A.1 — Install Simplification + Secret Store Plan

> Temporary planning document. Removed at Phase D.

## Why this round

Round 10A landed comprehensive package installation, but two design choices
diverged from the platform's UX intent:

1. **Install was over-secured** for the indie/community-package context.
   Defaults required GPG-signed tags and blocked on conformance failures —
   appropriate for enterprise supply chains, mismatched for the casual
   GitHub URL paste flow that ST community packages live in. Cargo, npm, pip
   don't require signatures by default; we shouldn't either.

2. **Secret management forced env-only**, requiring users to set
   `OPENAI_API_KEY` before launch. ST originals provide an in-app API
   Connections panel where users paste their key once and the host stores it
   locally. Yggdrasil's contract was right (`secret_ref:`), but the only
   resolver was env-based, leaving no path for "save my key in the app".

This round fixes both without changing the kernel.

## Phases

### Phase A — Install simplification (~1 day)

Make defaults match the cargo/npm/pip baseline:

- **GPG signing**: `allow_unsigned` flips from opt-in to **default true**.
  Field renamed `require_signed` (default false) for clarity. Users with
  signed packages opt in; everyone else just installs.
- **Conformance gating**: `ignore_conformance` becomes `strict_conformance`
  (default false). Conformance failures become **warnings printed to stderr**
  by default, blocking only when `--strict` is set.
- **Consent prompt**: Replace 4-section dialoguer rendering with a single
  one-line `Confirm`. New permissions still listed but without category
  headers + colored separators. Match cargo/npm `[y/N]` style.
- **CLI flags**: `--allow-unsigned` removed (default behavior); add
  `--require-signed` and `--strict` for users who want stricter behavior.
- **Conformance cases**: Update affected cases to test new defaults.
  Existing `ignore_conformance` test renamed to `strict_conformance` test.

Files:
- `crates/ygg-runtime/src/inproc/install_lab.rs` — flip defaults, rename fields
- `crates/ygg-cli/src/commands/install.rs` — flag rename, new defaults
- `crates/ygg-cli/src/install/consent.rs` — single-line prompt
- `crates/ygg-cli/src/conformance/install_lab.rs` — case updates
- `crates/ygg-cli/tests/install_commands.rs` — initializer updates

### Phase B — Secret store package (~2-3 days)

Add `secret_ref:store:NAME` resolution backed by an encrypted local store.

**B1: New `official/secret-store-lab` capability package**
- `secret-store.put_secret(name, value)` — store an encrypted secret
- `secret-store.get_secret(name)` — return raw value (privileged; only
  callable by the host secret resolver, not by ordinary packages)
- `secret-store.list_secrets()` — return names only, never values
- `secret-store.delete_secret(name)` — remove
- `secret-store.has_secret(name)` — boolean check (for UI)

Encryption design:
- Store at `~/.yggdrasil/secrets.dat` (env-overridable via paths.rs)
- Format: `age` (rage crate, MIT/Apache, AGPL-compatible) — modern,
  authenticated, audited
- Master key resolution order:
  1. OS keyring (via `keyring` crate) — primary path
  2. `~/.yggdrasil/secret-store.key` file (0600 perms) — fallback
  3. Generate new key on first put_secret, persist to keyring then file
- File format: age-encrypted JSON `{ "secrets": { "NAME": "value", ... } }`
- File permissions: 0600 on Unix
- Atomic writes (tmp + rename)

**B2: Extend host secret resolver**

Add `StoreSecretResolver` next to `EnvSecretResolver`:
- Resolves `secret_ref:store:NAME` by calling `secret-store.get_secret`
- Resolves via direct file read if the secret-store-lab package isn't
  loaded (resolver shouldn't depend on package being active)
- Same fail-closed contract: missing → error, never leak value

Add `CompositeSecretResolver` that chains multiple resolvers:
- Try store resolver first for `secret_ref:store:`
- Try env resolver for `secret_ref:env:`
- Fall through with clear error per ref pattern

**B3: Profile config integration**
- Profile YAML gets `secret_store: { enabled: bool, path: string? }`
- Default profile sets `secret_store.enabled: true`
- Host wiring: when serving, build `CompositeSecretResolver` if store enabled

**B4: Conformance**
- 5+ cases for secret-store-lab capabilities
- 3 cases for resolver: store path, env path, missing → fail-closed
- 1 case for raw secret leak prevention in error messages

Files:
- `crates/ygg-runtime/Cargo.toml` — add `age`, `rage`, `keyring`, deps
- `crates/ygg-runtime/src/inproc/secret_store_lab.rs` — new module
- `crates/ygg-runtime/src/secret.rs` — add `StoreSecretResolver`,
  `CompositeSecretResolver`
- `crates/ygg-core/src/secret_ref.rs` — extend `is_valid_ref` to accept
  `secret_ref:store:NAME`
- `crates/ygg-core/src/paths.rs` — add `secret_store_path()`
- `packages/official/secret-store-lab/manifest.yaml` — new package
- `profiles/forge-alpha.yaml` — autoload + secret_store config
- `crates/ygg-cli/src/conformance/secret_store.rs` — new file with cases

### Phase C — YdlTavern API Connections wired (~1-2 days)

YdlTavern's existing `APIConnectionsDrawer.tsx` becomes functional.

- UI form: provider picker (OpenAI/Anthropic/Gemini/...) + key input + save
- Save calls `secret-store.put_secret("OPENAI_API_KEY", value)` via host
  protocol (kernel.v1.capability.invoke)
- List shows configured providers with masked indicator (✓ stored)
- Delete removes via `secret-store.delete_secret`
- Connection test button: invokes a small probe through outbound.execute
  with `secret_ref:store:OPENAI_API_KEY` to verify
- Model calls in YdlTavern use `secret_ref:store:OPENAI_API_KEY` instead
  of env

The provider profile config already supports `secret_ref:` strings; this
just changes which prefix the UI emits when saving.

Files:
- `packages/ydltavern-surface/src/components/shell/drawers/APIConnectionsDrawer.tsx`
  — full form
- `packages/ydltavern-surface/src/state/secrets.ts` — new helper for
  store/list/delete via host RPC
- `packages/ydltavern-engine/...` — switch model call provider profile
  default from env to store reference (maintain env path as fallback)
- Tests in surface + engine

### Phase D — Docs convergence (~half day)

- New bilingual `docs/guides/SECRET_MANAGEMENT.{md,en.md}` covering:
  - Three resolver paths: env / store / future vault
  - When to use which
  - Security properties of each
  - Migration from env to store
- Update `docs/guides/PACKAGE_INSTALLATION.{md,en.md}`:
  - Default behavior: unsigned packages allowed, conformance warnings
  - When to opt into `--require-signed` / `--strict`
  - Match the simpler examples
- Update `docs/spec/KERNEL_V1_CONTRACT.{md,en.md}`:
  - Add `secret_ref:store:` to the resolver examples
- Update `README.{md,en.md}`:
  - Mention API Connections in YdlTavern works in-app
- Update `docs/ALPHA_STATUS.{md,en.md}`:
  - Mark Round 10A.1 complete
  - Update install defaults description
- Update YdlTavern `README.{md,en.md}` and `docs/...`:
  - API Connections drawer documented
  - secret_ref:store usage documented
- Delete `docs/INSTALL_SIMPLIFY_PLAN.md` (this file)
- Final cross-repo validation

## Wave plan

```
Wave 1: A ∥ B          (independent: A is CLI/install, B is secrets)
Wave 2: C              (depends on B)
Wave 3: D              (depends on A, B, C; final)
```

## Push cadence

Each wave commits + pushes when complete.

## Constraints

- AGPL-3.0 compatible (age, keyring are MIT/Apache, OK)
- Kernel still no new methods, no new ontology
- Backward compat: existing `secret_ref:env:` continues to work unchanged
- Existing CLI flags `--allow-unsigned` / `--ignore-conformance` removed
  cleanly (pre-release, no deprecation)
- New `--require-signed` / `--strict` opt-in flags added
- Existing test fixtures may need updates for new defaults
