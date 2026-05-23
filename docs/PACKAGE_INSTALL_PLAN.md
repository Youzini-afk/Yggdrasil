# Round 10A (I-track) — Package Installation Foundation Plan

> Temporary planning document. Removed at I10 once docs converge.

## Mission

Make "user downloads Yggdrasil → installs a package from a GitHub URL"
a real, secure, reproducible flow. While preserving the kernel/capability
boundary established in Round 9: the kernel knows nothing about git or package
managers; install machinery lives in ordinary capability packages.

## Target user flows

```bash
# Simple case
yg install github.com/user/yggdrasil-package

# Pinned version (recommended)
yg install github.com/user/yggdrasil-package#v1.2.0

# With signature verification (allowlist of public keys)
yg install github.com/user/yggdrasil-package#v1.2.0 --verify-signed-by ~/.yggdrasil/keys/

# Local path (development)
yg install ./packages/my-package

# Other operations
yg uninstall <package-id>
yg list-installed
yg update                           # check upstream for all installed
yg update <package-id>              # update specific
yg lockfile --check                 # verify lockfile matches store

# Profile-aware
yg install <url> --profile alpha
```

## Architecture: 4 layers

### Layer 1 — Kernel primitives (NO new kernel methods)

The kernel does not gain `package.install`, `git.clone`, or any related
ontology. All install machinery uses existing primitives:

- `kernel.v1.outbound.execute` — HTTPS transport, including git smart-http
- `kernel.v1.outbound.stream` — for streamed git pack data
- `permissions.network.declarations` — packages declare allowed git hosts
- `permissions.filesystem.write` — packages declare allowed write paths
- `kernel.v1.package.load` — still takes a local path; orchestration stages
  to local store first, then loads

The ONE manifest schema change: a new `requires` field for first-class package
dependency declaration. This is data, not protocol, and parallels the existing
`consumes` field for capabilities.

### Layer 2 — Three new official capability packages

#### official/git-tools-lab

Pure-Rust git operations over HTTPS via `kernel.v1.outbound.execute`. Uses
`gix` (Apache-2.0/MIT) as the implementation library, AGPL-compatible.

Capabilities:
- `git.resolve_ref` — input: `{ remote_url, ref }`, output: `{ commit_sha, ref_kind }`
- `git.fetch_refs` — input: `{ remote_url }`, output: `{ refs: [{name, sha}] }`
- `git.fetch_tree` — input: `{ remote_url, commit_sha, dest_dir }`, output:
  `{ files_written, total_bytes, tree_hash }`. Atomic: writes to tmp + renames.
- `git.read_signed_tag` — input: `{ remote_url, tag }`, output: `{ tag_object,
  pgp_signature?, signed_data }`

Manifest declares:
- `permissions.network.declarations` for `github.com`, `gitlab.com`,
  `codeberg.org`, etc.
- `permissions.filesystem.write` for the staging directory

#### official/integrity-lab

Hashing and signature verification primitives. Uses `sequoia-openpgp` for GPG
verification, deferring sigstore until standard Git tag convention exists.

Capabilities:
- `integrity.compute_tree_hash` — input: `{ dir }`, output: `{ sha256 }`
  (deterministic, sorted, no extended attributes)
- `integrity.compute_manifest_hash` — input: `{ manifest_path }`, output:
  `{ sha256 }` (canonical normalize before hash)
- `integrity.verify_gpg_signature` — input: `{ data, signature, public_keys: [armored] }`,
  output: `{ verified: bool, key_fingerprint?, error? }`

Manifest declares:
- `permissions.filesystem.read` for the directory being hashed

#### official/install-lab

Orchestration. Composes git-tools-lab + integrity-lab + conformance kit.

Capabilities:
- `install.resolve_plan` — input: `{ root_url, lockfile?, allow_unsigned? }`,
  output: `{ packages: [...], permissions_summary, signature_summary, integrity_hashes }`.
  Recursive: fetches each transitive dep's manifest, builds full plan.
- `install.execute_plan` — input: `{ plan, signed_consent }`, output:
  `{ installed: [...], lockfile_updates }`. Atomic: writes everything to tmp,
  renames, then updates profile.
- `install.uninstall` — input: `{ package_id, profile? }`, output:
  `{ removed_from_profile, store_paths_orphaned }`. Does NOT delete from store
  (immutable; orphaned only).
- `install.list_installed` — input: `{ profile? }`, output: `{ packages: [...] }`
- `install.check_lockfile` — input: `{ profile? }`, output: `{ ok, drift: [...] }`

Manifest declares:
- Consumes git-tools-lab and integrity-lab
- `permissions.filesystem.write` for `~/.yggdrasil/store/`,
  `~/.yggdrasil/profiles/`
- Network not needed directly — delegates to git-tools-lab

### Layer 3 — CLI commands (host-level orchestrator)

The CLI is the host operator and can call kernel methods directly. It composes
install-lab capabilities into user-facing commands.

```rust
// Command flow for `yg install <url>`:
// 1. Parse URL → normalize to https://host/owner/repo + ref
// 2. Read existing lockfile (if any) for current profile
// 3. Invoke install-lab.resolve_plan
// 4. Show plan to user:
//    - Packages to install (root + transitive)
//    - Total permissions requested
//    - Signature status (signed/unsigned/mismatch)
//    - Integrity hashes
// 5. If --yes flag absent, prompt for consent
//    - Highlight new capabilities not previously granted
//    - Refuse if user declines
// 6. Invoke install-lab.execute_plan with signed consent
// 7. Update lockfile
// 8. Optionally invoke kernel.v1.package.load to activate immediately
//    (or defer until next host serve restart)
```

### Layer 4 — Manifest + Lockfile

#### Manifest extension

Add to `ygg-core::manifest::PackageManifest`:

```yaml
requires:
  - id: "official/composition-lab"
    source: "internal"           # or "git", "local"
    version: ">=1.0.0"           # semver constraint
  - id: "third-party/cool-tool"
    source: "git"
    url: "https://github.com/user/cool-tool"
    ref: "v1.2.0"
    # optional integrity (lockfile fills this; manifest only sets minimum)
    minimum_signed_by: ["FA9C..."]  # GPG fingerprint allowlist
```

#### Lockfile (TOML)

Location: `~/.yggdrasil/profiles/<name>.lock.toml`

```toml
schema = "yggdrasil.lock.v1"
profile = "default"
generated_at = "2026-05-23T12:00:00Z"
manifest_hash = "sha256:abc..."

[[package]]
id = "third-party/cool-tool"
version = "1.2.0"
source = "git"
url = "https://github.com/user/cool-tool"
ref = "v1.2.0"
commit = "abcdef1234567890abcdef1234567890abcdef12"
tree_hash = "sha256:1234..."
manifest_hash = "sha256:5678..."
signed = true
signed_by = "FA9C5BC2B71E1FF20BD63A2F3D8E9A1B2C4D5E6F"
installed_at_store = "~/.yggdrasil/store/sha256-9abc..."
granted_capabilities = ["model.live_call"]
granted_network = ["api.openai.com"]
granted_secrets = ["OPENAI_API_KEY"]

[[package.requires]]
id = "official/composition-lab"
constraint = ">=1.0.0"
resolved_to = "1.0.5"
```

### Layer 5 — Filesystem convention

```text
~/.yggdrasil/                         # XDG-overridable
├── store/                            # immutable, content-addressed
│   ├── sha256-9abc.../              # one package per hash
│   │   ├── manifest.yaml
│   │   ├── package.py / *.rs / etc.
│   │   └── ...
│   └── sha256-def0.../
├── profiles/                         # mutable, per-user
│   ├── default.yaml                 # autoload list
│   ├── default.lock.toml
│   ├── alpha.yaml
│   └── alpha.lock.toml
├── keys/                             # GPG public keys for verification
│   └── trusted-keys.asc
└── cache/
    └── git/                         # git fetch cache (refs, packfiles)
```

Use `dirs` crate to resolve `~/.yggdrasil` (respects `XDG_DATA_HOME`).

## Phases

### I0 — This plan + push
- Write `docs/PACKAGE_INSTALL_PLAN.md` (this file)
- Push to Yggdrasil

### I1 — Manifest extension + lockfile schema (foundation)
- Add `requires` field to `PackageManifest` in `crates/ygg-core/src/manifest.rs`
- New struct: `PackageDependency` (id, source, url?, ref?, version, minimum_signed_by?)
- New module: `crates/ygg-core/src/lockfile.rs` — `Lockfile` + `LockEntry` types
- TOML serialization round-trip tests
- Update `docs/spec/v1/schemas/manifest.schema.json` (regenerate)
- Update `docs/spec/v1/schemas/permission-set.schema.json` if needed
- Add to `KERNEL_V1_CONTRACT.md` mention of `requires` field

### I2 — official/git-tools-lab
- New package directory: `packages/official/git-tools-lab/`
  - manifest.yaml: declares 4 capabilities, network/filesystem permissions
- New inproc impl: `crates/ygg-runtime/src/inproc/git_tools_lab.rs`
  - Uses `gix` for git operations
  - Routes HTTPS through `kernel.v1.outbound.execute` for transport
  - Atomic writes (tmp dir + rename)
- Add `gix` dependency to `crates/ygg-runtime/Cargo.toml`
- Add to inproc registration mod.rs
- Conformance cases for each capability
- Add to `profiles/forge-alpha.yaml` autoload

### I3 — official/integrity-lab
- New package directory: `packages/official/integrity-lab/`
- New inproc impl: `crates/ygg-runtime/src/inproc/integrity_lab.rs`
  - SHA256 tree hash (deterministic ordering)
  - Manifest canonical-normalize + hash
  - GPG verification via `sequoia-openpgp`
- Add `sha2` and `sequoia-openpgp` dependencies
- Conformance cases
- Add to `profiles/forge-alpha.yaml`

### I4 — official/install-lab
- New package directory: `packages/official/install-lab/`
- New inproc impl: `crates/ygg-runtime/src/inproc/install_lab.rs`
- Capabilities: resolve_plan, execute_plan, uninstall, list_installed, check_lockfile
- Composes git-tools-lab + integrity-lab + project-intake-lab + conformance kit
- Transitive dependency resolution with cycle detection
- Atomic install pattern (write to staging, rename to store, atomic profile update)
- Conformance cases including transitive resolution
- Add to `profiles/forge-alpha.yaml`

### I5 — CLI commands
- `crates/ygg-cli/src/commands/install.rs` — `yg install`
- `crates/ygg-cli/src/commands/uninstall.rs` — `yg uninstall`
- `crates/ygg-cli/src/commands/list_installed.rs` — `yg list-installed`
- `crates/ygg-cli/src/commands/update.rs` — `yg update`
- `crates/ygg-cli/src/commands/lockfile.rs` — `yg lockfile`
- Wire into `cli.rs` and `main.rs`
- URL parsing helper (accepts `github.com/x/y`, `https://...`, `./path`)
- Each command invokes install-lab via runtime

### I6 — Permission consent prompts (CLI)
- `crates/ygg-cli/src/install/consent.rs` — interactive prompt
- Diff against existing granted authority in lockfile
- Highlight only NEW or expanded permissions
- Skip prompt with `--yes` flag (CI mode)
- JSON output for non-interactive flow with `--format json`
- Tauri/UI integration deferred (just CLI for now)

### I7 — Filesystem convention (~/.yggdrasil)
- `crates/ygg-core/src/paths.rs` — central path resolution
- Uses `dirs` crate, respects `YGG_DATA_DIR` env override
- Functions: `data_dir()`, `store_dir()`, `profiles_dir()`, `keys_dir()`, `cache_dir()`
- All install-lab paths go through this module
- Default profile creation if missing
- Migration story: detect old layout, suggest action (none for now since pre-release)

### I8 — Conformance kit integration
- `yg conformance package --transitive` — also check transitive deps
- Install flow tests that the conformance kit is invoked during install
- A failed conformance check on a dependency aborts install (with override)
- Integration in install-lab.resolve_plan

### I9 — End-to-end test
- Fixture: `examples/packages/install-fixture-root/` (declares requires)
- Fixture: `examples/packages/install-fixture-dep/` (transitive dep)
- Conformance case: install root, verify transitive installed, verify lockfile,
  uninstall, verify cleaned up
- Use local file path for fixture (no real GitHub fetch in CI)
- Optional: opt-in real GitHub smoke test gated by `YGG_GIT_INSTALL_REAL_TESTS=1`

### I10 — Docs convergence
- New `docs/guides/PACKAGE_INSTALLATION.{md,en.md}` (bilingual)
- Update `docs/spec/KERNEL_V1_CONTRACT.{md,en.md}` (mention `requires` field)
- Update `docs/ALPHA_STATUS.{md,en.md}`
- Update `docs/spec/CONFORMANCE_MATRIX.{md,en.md}`
- Update `docs/roadmap/NEXT_STEPS.{md,en.md}` (mark Round 10A complete)
- Update `README.{md,en.md}` (mention yg install in feature list)
- Delete `docs/PACKAGE_INSTALL_PLAN.md`
- Final cross-repo validation

## Wave plan

```
Wave 1: I1                               (foundation, must be first)
Wave 2: I2 ∥ I3                          (independent capability packages)
Wave 3: I4                               (depends on I2 + I3)
Wave 4: I5 ∥ I7                          (CLI + filesystem layout independent)
Wave 5: I6 ∥ I8                          (consent + conformance integration)
Wave 6: I9                               (e2e test, depends on prior waves)
Wave 7: I10                              (docs + delete plan)
```

## Push cadence

Each wave commits + pushes when complete. Final wave deletes this plan.

## Edge cases handled

- Circular deps in transitive resolution → detect, refuse with cycle path
- Already-installed at different version → prompt for upgrade/override
- Repo without manifest → refuse, "not a Yggdrasil package"
- Signature mismatch → refuse with explicit error, abort install
- Network unavailable → clear error, suggest --offline if cached
- Conformance fail on transitive dep → abort with `--ignore-conformance` override
- Mid-install crash → tmp dir orphaned, no profile change (atomic guarantee)
- Same package referenced by two deps with different versions → resolve to
  highest compatible, warn if no overlap

## Out of scope (defer)

- Sigstore keyless verification (no Git tag convention yet)
- Tauri UI for install (CLI prompts only this round)
- Central marketplace registry (against platform philosophy)
- Auto-update daemon (manual `yg update` only)
- Binary package distribution (source/git-only this round)
- Cross-profile package sharing semantics (each profile has own store ref)

## Constraints reminder

- Kernel: no new methods, no new ontology, no git knowledge
- New `requires` is a manifest field (data), not protocol
- All git/install machinery in capability packages with declared permissions
- Lockfile is package-owned data, not kernel-owned
- Store is filesystem convention, not kernel state
- AGPL-3.0 compatible (gix Apache/MIT, sequoia LGPL with Yggdrasil compat note)
- Conformance kit integrated at install time (verify before activate)

## Estimated work

8-10 days self-driven across the 7 waves, with subagent dispatch for bounded
implementation tasks.
