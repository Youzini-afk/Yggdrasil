# Round 10A.2 — Steam-Game Project Model Plan

> Temporary planning document. Removed at Wave 5 once docs converge.

## Vision

Yggdrasil's user-facing model becomes Steam-like: a platform that holds many
projects, each appearing as a card on Home, each runnable independently. The
platform provides shared infrastructure (packages, platform-level secrets, base
UI), and projects are the play/use entry points.

```text
Yggdrasil platform
  ↓ holds platform-level resources (shared)
  ├─ Installed capability packages (model-provider-lab, persona-lab, ...)
  ├─ Platform secret store (~/.yggdrasil/secrets.dat)
  └─ Master encryption key
       ↓
Projects (Home cards, independently runnable)
  ├─ YdlTavern              (yggdrasil_native: has project.yaml, uses Ygg packages)
  ├─ Coding Agent (future)  (yggdrasil_native)
  ├─ Some Tool (external)   (external_workspace: plain git repo, agent-driven)
  └─ Wrapped CLI (external) (external_wrapped: adapter package wraps it)
```

Soft isolation, not hard tenant isolation:
- Each project has its own data directory (state, sessions, settings, secrets)
- Default secret resolution falls back project → platform (convenience first)
- One `yg host serve` runs many projects concurrently; status pill on each card
- Packages are platform-shared; no per-project package duplication
- No multi-user/access-control complexity

## Critical bug discovered before this round

`crates/ygg-cli/src/commands/host.rs::runtime_config_from_profile` builds
`RuntimeConfig::default()` which uses `DenyAllSecretResolver`. It wires
outbound policy/executor but never installs `CompositeSecretResolver(env+store)`.
Round 10A.1's UI saves keys correctly, but `ygg host serve --profile` fails to
resolve them at outbound time. **Wave 1 fixes this before any project work.**

## Wave structure

```
Wave 1: P0 resolver wiring fix                  (~1 day, blocking)
Wave 2: Project as first-class runtime concept  (~3-4 days)
Wave 3: Install detection + project lifecycle   (~2-3 days)
Wave 4: Home surface for project cards          (~1-2 days)
Wave 5: YdlTavern integration + docs converge   (~1-2 days)
```

Total: ~8-12 days self-driven with subagent dispatch.

## Wave 1 — P0 resolver wiring fix

**Goal**: `ygg host serve --profile <p>` produces a Runtime that actually
resolves `secret_ref:env:*` and `secret_ref:store:*`. Without this, Round 10A.1
is functionally broken.

### Changes

1. `crates/ygg-cli/src/commands/host.rs::runtime_config_from_profile`:
   - Build `EnvSecretResolver` from manifest-declared secret refs (or from
     a new `secret_resolver:` section in HostProfile YAML, allowlist-controlled).
   - Build `StoreSecretResolver` for the platform store path.
   - Wrap both in `CompositeSecretResolver` and install via
     `RuntimeConfig::with_secret_resolver(...)`.

2. `crates/ygg-cli/src/cli.rs::HostProfile`:
   - Add optional `secret_resolver` section:
     ```yaml
     secret_resolver:
       env_allowlist: ["OPENAI_API_KEY", "ANTHROPIC_API_KEY", ...]
       store_enabled: true
     ```
   - Default: `store_enabled: true`, env allowlist empty (deny env unless
     explicitly listed; matches current security model).

3. `profiles/forge-alpha.yaml`:
   - Add `secret_resolver: { store_enabled: true }`.

4. Tests:
   - Integration test that profile-loaded host actually resolves both ref types.
   - Conformance case: profile with store disabled denies `secret_ref:store:*`.

### Validation

- `cargo test -p ygg-cli` passes
- `cargo run -p ygg-cli -- conformance` passes (398 → 399 with new case)
- Smoke: write a secret via secret-store-lab, then verify outbound.execute
  with `secret_ref:store:NAME` actually injects the value (in fake executor mode)

## Wave 2 — Project as first-class runtime concept

**Goal**: A `ProjectDescriptor` exists in `ygg-core`, projects have a
durable identity, per-project data directories work, and
`secret_ref:project:NAME` resolves with platform-level fallback.

### Architectural decisions

**ProjectDescriptor lives in `ygg-core`**, distinct from `CompositionDescriptor`
(which stays in ygg-cli for static validation). Composition validates a
package set; Project is a runtime instance with state.

**project_type**:
- `yggdrasil_native` — has project.yaml, manifest references Ygg packages
- `external_wrapped` — external project with adapter package wrapping it
- `external_workspace` — external project running as agent workspace

**Project data layout**:
```
~/.yggdrasil/projects/<project_id>/
├── project.yaml              # ProjectDescriptor (descriptor copy + state)
├── secrets.dat               # age-encrypted, same master key as platform
├── sessions/                 # per-project session event store backend
├── state/                    # package-owned state files
└── lockfile.toml             # which packages this project pinned
```

`project_id` rules:
- Format: `<owner>__<name>` slug + 8-char short hash for collision safety
  (e.g., `youzini-afk__YdlTavern__a1b2c3d4`)
- Filesystem-safe; no `/`, no `..`, no shell-special chars
- Stable across composition upgrades

**Secret resolver chain** (the soft-fallback model user requested):
```
secret_ref:env:NAME      → EnvSecretResolver only (allowlist controlled)
secret_ref:store:NAME    → StoreSecretResolver only (platform)
secret_ref:project:NAME  → ProjectStoreSecretResolver(active_project_id)
                            → falls back to StoreSecretResolver if not found
                              and project.yaml secret_policy.fallback_to_platform: true (default)
                            → fail-closed otherwise
```

The fallback is policy-controlled per project, not a global default. Project
authors who want strict isolation set `fallback_to_platform: false`.

**project_id flow** (for resolver context):
- Sessions get `metadata.project_id` (existing field; new convention)
- `ProtocolContext` gains `session_id: Option<String>` (new field)
- Outbound dispatch reads session_id → session.metadata.project_id → resolver scope
- Calls without session context (host admin ops) → no project scope → project: refs fail-closed
  unless host is configured with a default active project

### Changes

1. **New `ygg-core::project` module** (~200 LOC):
   - `ProjectDescriptor`, `ProjectType` enum, `SecretPolicy`, `ProjectState`
   - Validation: id format, paths, type-specific required fields
   - YAML/JSON serialization with round-trip tests

2. **Extend `ygg-core::paths`**:
   - `projects_dir() -> ~/.yggdrasil/projects/`
   - `project_dir(id) -> projects_dir().join(id)`
   - `project_secret_store_path(id) -> project_dir(id).join("secrets.dat")`
   - `project_lockfile_path(id) -> project_dir(id).join("lockfile.toml")`
   - `archived_projects_dir() -> projects_dir().join(".archived")`
   - id-safety helper that rejects unsafe inputs

3. **Extend `ygg-core::secret_ref`**:
   - `is_project_backed_ref(s)` and `extract_project_name(s)` helpers
   - Manifest validation accepts `secret_ref:project:NAME`

4. **Extend `ygg-runtime::secret`**:
   - `ProjectStoreSecretResolver { active_project_id, fallback_to_platform, platform: StoreSecretResolver }`
   - `CompositeSecretResolver` learns to route `project:` to the project resolver
   - Active project id is set on RuntimeConfig (single default) and overridable per-call later

5. **Extend `ygg-core::session`**:
   - Document `metadata.project_id` convention (no schema break; metadata is
     already a string map)
   - Helper to set/get project_id

6. **Extend `ygg-runtime::protocol::ProtocolContext`**:
   - Add `session_id: Option<String>` field
   - Subprocess supervisor and HTTP transport propagate it
   - Existing call sites default to None (backward compatible inside this round)
   - SDK regeneration

7. **Extend outbound dispatch**:
   - `dispatch_outbound_execute` reads session_id from ProtocolContext
   - Looks up session.metadata.project_id
   - Passes it to runtime resolver

8. **Project registry in runtime**:
   - On host startup, scan `~/.yggdrasil/projects/*/project.yaml`
   - Build in-memory `ProjectRegistry`
   - State machine: Installed → Starting → Running → Stopping → Stopped → Failed
                    Stopped/Failed → Archived (via uninstall)

9. **`secret-store-lab` extension**:
   - New capability `put_project_secret { project_id, name, value }` (host-trusted only)
   - New capability `list_project_secrets { project_id }` (returns names only)
   - New capability `delete_project_secret { project_id, name }`
   - Validate caller has access to project (Wave 2: trust UI; later: principal check)

10. **Conformance**:
    - `secret_resolver.project_falls_back_to_platform`
    - `secret_resolver.project_no_fallback_when_disabled`
    - `secret_resolver.project_isolation_between_projects`
    - `secret_resolver.project_resolution_without_project_context_fails`
    - Project descriptor validation cases

### Validation

- 398 → ~404 conformance cases pass
- `cargo test -p ygg-core` covers ProjectDescriptor + paths
- `cargo test -p ygg-runtime` covers resolver chain + isolation
- Smoke: store project secret + platform secret with same name → project takes precedence
- Smoke: project without secret + platform with secret + fallback enabled → resolves to platform
- Smoke: project without secret + fallback disabled → fail-closed

## Wave 3 — Install detection + project lifecycle

**Goal**: `yg install` recognizes native vs external projects and registers
them appropriately. `yg project` subcommand exists with start/stop/list/status/uninstall.

### Changes

1. **Detection logic in `install-lab`**:
   - After fetch_tree, look for `project.yaml` at repo root
   - If found and it parses as ProjectDescriptor with type `yggdrasil_native`:
     → Native project. Register directly.
   - If found with type `external_*`: validate and register
   - If not found:
     → External project. Trigger wizard (CLI prompt for now).

2. **Wizard for external projects** (in CLI):
   ```
   The repository at github.com/user/some-tool does not declare itself as a
   Yggdrasil project. How do you want to use it?

     [1] Wrap with adapter (creates a Yggdrasil package that wraps the tool)
     [2] Open as workspace (run in agent-driven workspace; no wrapping)
     [3] Cancel

   Choose [1/2/3]:
   ```
   - Option 1: invokes adapter-generator-lab.generate_adapter_manifest_preview
     (already exists from Round 8) and creates an `external_wrapped` project
   - Option 2: creates an `external_workspace` project that points at the
     fetched directory; project-intake-lab + workspace-lab handle agent workflows
   - `--no-prompt` flag picks option 2 by default for CI/automation
   - `--wrap-as-adapter` and `--workspace-only` flags skip prompt

3. **`yg project` subcommand family**:
   - `yg project list` — list installed projects with id/title/type/state
   - `yg project start <id>` — transition to Running (loads project's packages,
     opens default session if applicable)
   - `yg project stop <id>` — graceful shutdown
   - `yg project status <id>` — detailed state, sessions, secrets summary
   - `yg project info <id>` — full descriptor + paths

4. **`yg uninstall` extension** (existing command):
   - Detect if uninstalling something registered as a project
   - Interactive prompt:
     ```
     Project YdlTavern (3 sessions, 2 project-scoped secrets) is installed.
     What about the project data?
       [1] Keep data (archive to ~/.yggdrasil/projects/.archived/<id>/, 30-day cleanup)
       [2] Delete data immediately
       [3] Cancel uninstall
     ```
   - `--keep-data` / `--delete-data` flags skip prompt for CI
   - Default if non-TTY without flag: keep-data (safe)
   - Archived projects can be restored via `yg project restore <id>` (future)

5. **Lockfile evolution**:
   - Per-project lockfile at `~/.yggdrasil/projects/<id>/lockfile.toml`
   - Records which packages this project pinned + their versions
   - `yg project lockfile <id> --check` for drift detection
   - The platform-level lockfile (existing) tracks platform-installed packages;
     project lockfile is additive

6. **Conformance**:
   - `install.detects_native_project_yaml`
   - `install.detects_external_no_manifest`
   - `project.lifecycle_install_start_stop_uninstall`
   - `project.uninstall_keep_data_archives`
   - `project.uninstall_delete_data_removes`
   - `project.list_reflects_state`

### Validation

- ~404 → ~410 conformance cases
- Demo: install local YdlTavern (after Wave 5 adds project.yaml), it appears
  in `yg project list` as `yggdrasil_native`, can be started/stopped
- Demo: install random GitHub repo without manifest, wizard prompts, both
  options create distinct project entries

## Wave 4 — Home surface for project cards

**Goal**: `clients/web` Home displays installed projects as cards, each with
title/icon/status pill/Play button. Clicking Play navigates to the project's
entry surface (for native) or appropriate launch action (for external).

### Changes

1. **Public protocol additions**:
   - `kernel.v1.project.list` returns installed projects
   - `kernel.v1.project.get` returns full descriptor + state
   - `kernel.v1.project.start` and `kernel.v1.project.stop`
   - `kernel.v1.project.status` (lightweight state poll)
   - All gated by host-only/admin principal in v1 (no public package access)
   - Schema files in `docs/spec/v1/schemas/methods/`
   - 4 method schemas + lifecycle event schemas (project.installed/started/stopped/uninstalled)

2. **clients/web Home rewrite**:
   - Fetch project list on mount via `/rpc kernel.v1.project.list`
   - Render grid of project cards: title, icon (or default), state pill, Play button
   - State pill colors: gray (Stopped), green (Running), yellow (Starting/Stopping), red (Failed)
   - Click Play on Stopped project → invoke `project.start` → on Running, navigate to entry
   - For native projects: entry is the project's `entry_surface_id` mounted in iframe
   - For external_workspace: entry is the agent workspace surface
   - For external_wrapped: entry is the adapter package's surface
   - Live status updates via SSE on `kernel/v1/project.*` events
   - "Install new project" card opens install dialog (URL input)

3. **Surface contribution**:
   - Home itself becomes a small surface bundle (no React deps; vanilla TS),
     served from `/clients/web/dist`
   - Existing surface bundles for projects mounted on demand

4. **Surface routing**:
   - `/` → Home (project shelf)
   - `/project/<id>` → mount project's entry surface
   - `/settings` → platform settings (API keys, packages, etc.)
   - URL state preserved on reload

5. **API key save dialog**:
   - From APIConnectionsDrawer in YdlTavern (already exists), the surface
     can now save to either platform store or current project store
   - Surface gets project context via `kernelClient.currentProject()`
   - UI radio: "Save for [Platform-wide / This project only]"

### Validation

- `clients/web` typecheck and build pass
- Demo: install local fixture project, visible on Home as card with correct state
- Demo: click Play, surface mounts, kernel events flow

## Wave 5 — YdlTavern integration + docs convergence

**Goal**: YdlTavern declares itself a yggdrasil_native project. Yggdrasil
docs explain the 3-tier model. Temporary plan removed.

### Changes (YdlTavern)

1. **Add `project.yaml` at repo root**:
   ```yaml
   schema_version: 1
   project:
     id: youzini-afk__YdlTavern
     title: YdlTavern
     description: SillyTavern-compatible AI roleplay surface for Yggdrasil.
     type: yggdrasil_native
     icon: ./assets/icon.png   # if available
     entry_surface_id: youzini-afk/ydltavern-surface/play
     packages:
       - packages/ydltavern-engine/manifest.yaml
       - packages/ydltavern-surface/manifest.yaml
       - packages/ydltavern-extensions/manifest.yaml
     secret_policy:
       fallback_to_platform: true
       require_per_project: []
   ```

2. **Update YdlTavern docs**:
   - README mentions install via `yg install github.com/Youzini-afk/Yggdrasil-Tavern`
   - ARCHITECTURE.md notes YdlTavern is a project on Yggdrasil's project shelf
   - COMPATIBILITY_MATRIX.md adds project model row

### Changes (Yggdrasil)

1. **New `docs/guides/PROJECT_MODEL.{md,en.md}`** (bilingual, ~250 lines):
   - The 3-tier model explained: kernel → packages → projects
   - Steam-game analogy
   - project.yaml format reference
   - Native vs external project types
   - Lifecycle (install → start → stop → uninstall)
   - Per-project data layout
   - Soft isolation model with platform fallback
   - secret_ref:project:NAME usage
   - Examples: declaring a native project; wrapping an external tool

2. **Update `docs/architecture/ARCHITECTURE.{md,en.md}`**:
   - Replace 2-tier framing with explicit 3-tier
   - Keep all kernel content-freedom invariants intact
   - Note projects are runtime/host concepts, not kernel concepts

3. **Update `docs/guides/SECRET_MANAGEMENT.{md,en.md}`**:
   - Add `secret_ref:project:NAME` to the resolver table
   - Explain platform fallback semantics
   - When to choose project vs platform vs env

4. **Update `docs/guides/PACKAGE_INSTALLATION.{md,en.md}`**:
   - Add native vs external detection flow
   - Add wizard description
   - Add `--wrap-as-adapter` / `--workspace-only` flags

5. **Update `docs/spec/KERNEL_V1_CONTRACT.{md,en.md}`**:
   - New `kernel.v1.project.*` methods
   - New `kernel/v1/project.*` event kinds

6. **Update `docs/ALPHA_STATUS.{md,en.md}`**:
   - Round 10A.2 section
   - 3-tier model reflected
   - Conformance count updated

7. **Update `docs/spec/CONFORMANCE_MATRIX.{md,en.md}`**:
   - All new conformance cases listed

8. **Update `docs/roadmap/NEXT_STEPS.{md,en.md}`**:
   - Mark 10A.2 complete
   - Round 10B/11 outlook unchanged

9. **Update `README.{md,en.md}`** (both repos):
   - Note Home shows project cards
   - Note YdlTavern is the first published project

10. **Delete temporary plan**:
    - `docs/PROJECT_MODEL_PLAN.md` removed
    - No residue elsewhere

### Validation

- All Yggdrasil tests pass; conformance ~410+ cases
- YdlTavern surface/engine tests pass; golden harness 20/20 maintained
- `yg install ./YdlTavern` registers as `yggdrasil_native` project
- Home shows YdlTavern card; click Play mounts surface
- API Connections dialog can save to project or platform

## Constraints

- AGPL-3.0 compatible
- Kernel content-free invariant maintained — `kernel.v1.project.*` is the only
  new namespace; project is treated as opaque container/scope, not as a content
  type
- Backward compatible inside this round: existing manifests, sessions, lockfiles
  continue working
- No multi-user/access-control complexity
- No marketplace/registry centralization
- secret_ref:env: and secret_ref:store: behavior unchanged
- Each wave commits + pushes when complete; final wave deletes plan + converges docs

## Out of scope (deferred)

- Multi-user / project membership / access control
- Project import/export bundles (sharing-lab handles bundle format already)
- Project-level package version pinning UI (lockfile exists, UI later)
- Project archival auto-cleanup beyond 30-day suggestion
- Cross-project session migration
- Project icon upload UI (uses path reference for now)
- Marketplace / discovery / project rating

## Risk register

- ProtocolContext schema change requires SDK regeneration; minor breaking
  for any third-party SDK consumers (none yet)
- Secret resolver fallback edge cases (no project context when expected)
  must fail-closed cleanly
- Multi-project concurrent state in single host serve needs careful state
  isolation in inproc packages — call out in PROJECT_MODEL guide

## Push cadence

```
Plan written + pushed                               (now)
Wave 1 complete + pushed                            (~1 day)
Wave 2 complete + pushed                            (~3-4 days after Wave 1)
Wave 3 complete + pushed                            (~2-3 days after Wave 2)
Wave 4 complete + pushed                            (~1-2 days after Wave 3)
Wave 5 complete + pushed (deletes this plan)        (~1-2 days after Wave 4)
Final report to user                                (after Wave 5 push)
```
