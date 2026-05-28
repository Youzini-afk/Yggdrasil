# Project Update & Bundle Freshness — Plan

Status: Phase 6 complete — CLI `ygg update` now routes through `official/install-lab/update_project` after the Phase 5 generic in-place update capability landed (2026-05-28). Phase 7 may begin.

## Oracle review amendments (2026-05-28)

The original plan was reviewed by @oracle before any phase started. Six must-fix items were folded into the relevant phases below:

- **Phase 1.1** uses the robust shim form: `globalThis.process ??= {}; globalThis.process.env ??= {}; globalThis.process.env.NODE_ENV ??= 'production';` — handles preexisting partial `process` objects.
- **Phase 1.2** picks SHA-256 prefix (first 16 hex chars of `bundle.mjs` content hash) — not mtime. Surface bundle URL also exposes `bundle_fingerprint` explicitly so the console can show it without parsing query strings.
- **Phase 1.4** routes `Cache-Control: no-cache, must-revalidate` only for `/surface-bundles/*`; `/assets/*` keeps default behavior. Header construction takes path/kind context.
- **Phase 2.2 + 2.3** schema migration runs at cold host start **before profile autoload**. Autoload must tolerate dangling store paths after migration: skip, emit `kernel/v1/host.autoload.skipped` diagnostics, continue. Lazy mid-run wipe is forbidden.
- **Phase 3** GC validates tree hash format (`sha256:[0-9a-f]{64}`) and only deletes canonical descendants of `<data_dir>/store/`. Walks the store directory itself rather than trusting raw lockfile strings; quarantines malformed entries.
- **Phase 4** declares a `network.read` side-effect in capability metadata since `check_for_updates` calls `git-tools-lab/resolve_ref` over the network.
- **Phase 5** uses a true update transaction: snapshot `{lockfile, profile, project descriptor, project registry, project dist}` before mutation; restore on failure. Project dist copy goes through temp dir + atomic rename (no destructive `remove_dir_all` before the new dist is guaranteed in place). Failed install/update store paths are deleted/quarantined so future installs do not skip-fetch a poisoned tree. Resolve once per project source, not once per package.
- **Phase 5** ExternalWrapped: `external_content: not_applicable`, but the adapter package (in `project.packages`) can still be updatable. Implementation defers the adapter path to a follow-up but documents the distinction.

Below the Phase sections reflect these amendments.

---

This is a temporary roadmap doc. It will be deleted after Phase 8 once the work has converged into the long-term docs (`ALPHA_STATUS`, `NEXT_STEPS`, install-lab spec, surface bundle hosting, console UI guide).

## Why this plan exists

A user reinstalled YdlTavern through the web shell. The project still failed to launch, with the surface iframe showing `process is not defined`. The kernel-level investigation (see prior research) revealed several intertwined platform-wide issues, not a YdlTavern-specific bug:

1. Surface bundle URLs are fixed paths (`/surface-bundles/projects/<id>/bundle.mjs`) with no cache busting and no `Cache-Control` headers. A new bundle on disk can still be hidden behind a stale browser/proxy cache.
2. `integrity-lab/compute_tree_hash` excludes `dist/` from the content hash. Upstream packages whose only change is the built `dist/` (very common for surface bundles) appear "identical" to the install layer; resolve_plan never produces a different `tree_hash`.
3. Store paths (`<data_dir>/store/<tree_hash>/`) are content-addressed but never garbage collected. Uninstall keeps orphans. Reinstalling the same URL hits the existing store entry and skips fetch entirely.
4. There is no `update_project` capability — only an internal CLI loop. Every web flow forces users into "uninstall + reinstall", which is both worse UX and ineffective due to (3).
5. The new project console is currently just a title + "Open project interface" button. When something does break, users have zero diagnostic surface.
6. Old YdlTavern bundles still reference `process.env.NODE_ENV` at runtime; even after upstream is rebuilt, deployed installs serve the old bundle.

These are platform-level gaps. The fix must be generic (all native projects, not just YdlTavern), preserve content-free kernel boundaries (install/update lives in `official/install-lab`, not in the kernel), and not depend on hand-rolled "delete and reinstall" by users.

## Decisions already locked in

- **Q1 = B**: include `dist/` in `compute_tree_hash`. This changes the hash algorithm — every existing store entry gets a new hash. Since we are pre-launch we accept a one-time wipe.
- **Q2 = wipe store entirely** on schema bump. We bump a store schema marker; on host start, mismatch wipes `<data_dir>/store/` (lockfiles preserved so URLs/refs are not lost; on next update_project the store is rebuilt). Pre-launch acceptable.
- All new capabilities live in `official/install-lab`. The kernel does not learn about update/refresh semantics.
- All Phase 1 fixes are generic surface-bundle fixes; they apply to every native project, not just YdlTavern.

## Boundaries

| Layer | Responsibility | This plan touches? |
|---|---|---|
| Kernel | Content-free protocol/dispatch | No new methods. Header changes only. |
| `official/install-lab` | Install / update / uninstall semantics | New capabilities: `check_for_updates`, `update_project`. Existing `uninstall` extended with refcount GC. |
| `official/integrity-lab` | Tree hash | `EXCLUDED_NAMES` change + schema version constant |
| `crates/ygg-service` | HTTP transport | `Cache-Control` header on `/surface-bundles/*` |
| `crates/ygg-runtime/src/runtime/protocol/surface.rs` | Bundle URL resolution | Append `?v=<fingerprint>` |
| `clients/web/public/surface-frame-bootstrap.js` | iframe loader | `process` shim, allow `?v=` query |
| `clients/web` | Console / install / project UI | Console diagnostics, update entry |
| `crates/ygg-cli/src/commands/update.rs` | CLI | Reroute to `update_project` capability |

## Phases

### Phase 0 — Plan + @oracle review (this doc)

- Write plan doc.
- @oracle review:
  - Are the boundaries correct? (kernel vs install-lab vs surface-bundle layer vs UI)
  - Is the phase ordering safe? (does any later phase depend on something earlier should establish?)
  - Are there missing risks (security, data integrity, multi-project state)?
  - Is the store schema-bump strategy sound?

Adjust plan based on review. Then start Phase 1.

### Phase 1 — P0 immediate fixes (generic surface bundle freshness)

Goal: any deployed install fails closed and not silently. Stale bundle cannot be served behind a fresh URL. Old bundles that depend on `process.env.NODE_ENV` boot anyway.

- 1.1 `clients/web/public/surface-frame-bootstrap.js`:
  - Top of file (before any user bundle import): robust partial-process shim:
    `globalThis.process ??= {}; globalThis.process.env ??= {}; globalThis.process.env.NODE_ENV ??= 'production';`
  - Update bootstrap test to assert presence.
- 1.2 Bundle URL cache busting:
  - `crates/ygg-runtime/src/runtime/protocol/surface.rs::try_resolve_via_project` and `try_resolve_via_dev_path`: compute `sha256(bundle.mjs)` and use the first 16 hex chars.
  - Emit both `bundle_url: "/surface-bundles/.../bundle.mjs?v=<fp>"` and explicit `bundle_fingerprint: "<fp>"`.
  - Stylesheet versioning is deferred until style-specific project bundles exist; current installed project bundles have no project stylesheet list.
- 1.3 Web shell allowlist:
  - `isAllowedAssetUrl` accepts query strings on same-origin allowed paths.
  - Bootstrap test asserts `?v=...` does not bypass the path allowlist.
- 1.4 HTTP cache headers:
  - `crates/ygg-service/src/lib.rs::public_static_headers`: emit `Cache-Control: no-cache, must-revalidate` for `/surface-bundles/*`, keep aggressive cache for fonts in `/assets/*` if applicable.
- 1.5 Tests:
  - Rust: bundle URL contains `?v=` and exposes `bundle_fingerprint` for dev bundle mode.
  - Web: bootstrap shim test, allowlist test.
  - Integration: bundle response carries `Cache-Control`.

Push after Phase 1.

### Phase 2 — `dist` in `tree_hash` (Q1=B) + store schema bump

Goal: any change to a built `dist/` is detectable; pre-launch one-shot store wipe.

- 2.1 `crates/ygg-runtime/src/inproc/integrity_lab.rs`:
  - Remove `"dist"` from `EXCLUDED_NAMES`.
  - Add `pub const TREE_HASH_SCHEMA_VERSION: u32 = 2;` (was implicit v1).
- 2.2 Store schema marker:
  - New `crates/ygg-runtime/src/inproc/install_lab/layout.rs` (or extend existing) function `ensure_store_schema(data_dir)`:
    - Reads `<store_dir>/.schema_version`.
    - If missing or != `TREE_HASH_SCHEMA_VERSION`:
      - Recursively delete every entry in `<store_dir>` except `.schema_version` itself.
      - Write new `<store_dir>/.schema_version` with current version.
      - Emit `kernel/v1/host.store_schema_migrated` event with `{from, to, wiped_paths_count}`.
- 2.3 Hook into runtime startup:
  - Wire the schema check into `Runtime::start_host()` or equivalent — must run before `execute_plan` is ever invocable.
- 2.4 Lockfile entries become "dangling" on schema bump (`installed_at_store` no longer exists). `update_project` (Phase 5) handles dangling → re-fetch.
- 2.5 Conformance:
  - Update fixtures that previously assumed `dist` was excluded.
  - Add a conformance case: install fixture with non-empty `dist/` produces a tree_hash that changes when `dist/` content changes.
- 2.6 Tests:
  - Tree hash differs when only `dist/` content changes.
  - Schema bump wipes store contents but preserves lockfiles.
  - Idempotent: re-running on already-bumped store is a no-op.

Push after Phase 2.

### Phase 3 — Store refcount GC

Goal: orphaned store entries removed automatically, no manual "clear cache" needed.

- 3.1 New helper in `install_lab/executor.rs` (or new module `gc.rs`):
  - `fn store_path_refcount(store_path: &Path, data_dir: Option<&str>) -> Result<usize>` reads every `profiles/*.lock` and counts `installed_at_store == store_path` matches.
  - `fn collect_orphaned_stores(data_dir) -> Vec<PathBuf>` walks `<data_dir>/store/`, filters where refcount == 0.
- 3.2 `uninstall` capability:
  - After lockfile/profile updates, run GC: for each path in `store_paths_orphaned`, if refcount across all profiles is 0, remove it.
  - Add `purge_orphaned_stores: bool` (default true) so behavior is opt-out.
  - Emit `kernel/v1/install-lab.store.gc` event with removed paths.
- 3.3 `execute_plan`:
  - After successful install (lockfile updated), GC any store paths now orphaned (e.g., when an upgrade replaces an old hash).
- 3.4 Tests:
  - Two profiles share a store_path → uninstall in one profile preserves it.
  - Last profile removes ref → store_path deleted.
  - update_project replacing a lockfile entry deletes the old store_path.

Push after Phase 3.

### Phase 4 — `official/install-lab/check_for_updates` (read-only)

Goal: a generic, project-aware staleness probe that the console and CLI can call without side effects.

- 4.1 Manifest declaration in `packages/official/install-lab/manifest.yaml`.
- 4.2 New module `install_lab/update_check.rs`:
  - Input: `{ data_dir, profile, project_id?, package_id? }` — when both omitted, scans every package; project_id filters to a project's packages; package_id targets one entry.
  - For each lockfile entry:
    - Determine `ProjectType` (look up project descriptor for native projects via `read_project_descriptor`).
    - `git` source → call `git-tools-lab/resolve_ref(url, ref)`, compare returned `commit_sha` to lockfile commit.
    - `local` source → recompute `compute_tree_hash` on the local path, compare to lockfile tree_hash.
    - External-wrapped/external-workspace adapters → `applicable: false`.
    - Lockfile entry whose `installed_at_store` is missing on disk → `dangling: true` (after Phase 2 schema bump).
  - Output:
    ```json
    {
      "results": [{
        "package_id": "...",
        "project_id": "...",   // null if not part of a registered project
        "applicable": true/false,
        "current_commit": "...",
        "current_tree_hash": "...",
        "upstream_commit": "...",   // git only
        "available": true/false,
        "dangling": true/false,
        "reason": "..."        // present when not applicable
      }]
    }
    ```
- 4.3 Conformance: positive case (commit changed upstream → available=true), negative case (no change → false), local source case, dangling case.
- 4.4 Schema export.

Push after Phase 4.

### Phase 5 — `official/install-lab/update_project` (in-place update)

Goal: a single capability the console and CLI call to update a project's packages atomically. Also rebuilds dangling entries (store wipe recovery).

- 5.1 Manifest declaration.
- 5.2 New module `install_lab/update_project.rs`:
  - Input: `{ project_id, profile?, data_dir?, force?, consent? }`.
  - Steps:
    1. Read project descriptor → identify project's package list.
    2. For each package: determine source via lockfile entry. Build a fresh resolve_plan input from upstream (git URL + ref, or local path).
    3. Run `resolve_plan` to produce a fresh plan.
    4. Diff: any package whose new tree_hash differs from current lockfile entry, OR whose lockfile `installed_at_store` is dangling, OR `force=true`, is included for execution.
    5. If nothing to do → return `{updated: false, reason: "up_to_date"}`.
    6. Otherwise: `execute_plan` with the new packages. New lockfile entries point at new store paths.
    7. Phase 3 GC removes orphaned old store paths.
    8. `copy_project_surface_dist` is unconditionally invoked (already true today) so `<data_dir>/projects/<id>/dist/` is rebuilt with new bundle.
  - Output:
    ```json
    {
      "updated": true/false,
      "project_id": "...",
      "changed_packages": [{
        "id": "...",
        "old_tree_hash": "...",
        "new_tree_hash": "...",
        "old_commit": "...",
        "new_commit": "..."
      }],
      "bundle_fingerprint": "...",
      "reason": "..."
    }
    ```
- 5.3 Atomicity:
  - Lockfile/profile writes use `atomic_write` (already present).
  - On execute_plan failure mid-update: revert lockfile snapshot.
  - The `register_project` step is a no-op for already-registered projects; project state stays valid throughout.
- 5.4 ProjectType handling:
  - `YggdrasilNative` → full path above.
  - `ExternalWrapped` / `ExternalWorkspace` → `updated: false, reason: "not_applicable"` (no error). When external content updates are wired later, this is where they plug in.
- 5.5 Tests:
  - Native git project, upstream commit changed → updated=true, store path changed, dist refreshed, old store GC'd.
  - Native git project, no upstream change → updated=false, reason=up_to_date.
  - `force=true` → re-fetch even when no change.
  - Dangling lockfile entry (Phase 2 wipe scenario) → re-fetch.
  - External project → updated=false, applicable=false.
- 5.6 Conformance: end-to-end with fixture project.

Push after Phase 5.

### Phase 6 — CLI `ygg update` routes through `update_project`

Goal: one implementation, one path. CLI and web consume the same capability.

- 6.1 `crates/ygg-cli/src/commands/update.rs`:
  - Replace the per-package fetch loop with calls to `update_project`.
  - With no args: scan `list_installed`, group by project, call `update_project` per project. Packages not associated with any project (rare) call a still-existing `update_package` shortcut **only if** there is one — otherwise they go through the project that owns them. (Pre-launch: every package is part of a project, so this branch is acceptable.)
  - `--check-only` calls `check_for_updates` and prints results without executing.
- 6.2 Tests:
  - Pure args parsing (already exists).
  - Smoke: spinning a fixture project, updating, verifying lockfile changes.

Push after Phase 6.

### Phase 7 — Project console diagnostics + update entry

Goal: the console (the in-shell project tab) is no longer just a card with a button. When something breaks the user can see what.

- 7.1 Console data sources (no new backend APIs):
  - `kernel.v1.project.get` → state, type, packages, entry_surface_id, running_session_id, storage_summary.
  - `kernel.v1.package.list` filtered by project's package list → state, last_failure, capability_count.
  - `kernel.v1.surface.resolve_bundle` → bundle URL + fingerprint.
  - `kernel.v1.event.list(session=running_session_id)` → recent project events with kind filter.
  - `kernel.v1.capability.invoke("official/install-lab/check_for_updates", {project_id})` → updates badge.
- 7.2 Layout (designer review later):
  - Header: existing back / status / stop / disabled audit/more.
  - Body: replace single placeholder card with three sections:
    - "Project interface" (open in tab + popup-blocked guidance + bundle fingerprint + last resolve time).
    - "Packages" (list of project's packages with state pill, last failure, package logs link).
    - "Activity" (last N events from the running session, kind-filtered).
    - "Updates" (status from check_for_updates, "Update project" button → calls update_project, surface progress + success/error toast).
- 7.3 Refresh diagnostics button.
- 7.4 Locale labels (en + zh-CN).
- 7.5 @designer review for polish + UX dead-ends.
- 7.6 Tests:
  - Project frame parses real diagnostics inputs.
  - Update action calls capability and reflects result.

Push after Phase 7.

### Phase 8 — Doc convergence

- 8.1 Delete `docs/roadmap/PROJECT_UPDATE_AND_BUNDLE_FRESHNESS.md`.
- 8.2 Update `docs/ALPHA_STATUS.md` / `.en.md`:
  - Mention `check_for_updates`, `update_project`, store GC, tree_hash schema v2.
  - Bump conformance count.
- 8.3 Update `docs/roadmap/NEXT_STEPS.md` / `.en.md`:
  - Move "project update" from open issues to landed.
  - Add follow-ups (UI iframe error roundtrip, multi-profile update, external project update wiring).
- 8.4 Update install-lab capability spec doc (or section in capability package doc).
- 8.5 Update surface hosting / web README to mention cache busting + `process` shim policy.

Push after Phase 8.

## Validation strategy

Each phase ends with:
- `cargo check --workspace` (or `cargo check -p <crate>` if container limits binary builds).
- `cargo test -p <relevant crates>`.
- `cargo run -p ygg-cli -- conformance` (full named cases).
- `./scripts/validate-schemas.sh` (when methods or events change).
- Web: `npm run check`, `npm test`, `npm run build` under `clients/web/`.
- For Phase 7: @designer review for UX dead-ends; @oracle review of console contract.

## Out of scope (explicit)

- Service worker. Cache-Control + query string is sufficient at the current deployment scale.
- Cross-tab broadcast of iframe `mount.error` back to the console.
- Multi-version coexistence in the store (single tree_hash per package version).
- External project content updates (only the kernel/install integration). Those packages can later expose their own update capability.
- Migrating existing user data (pre-launch).

## Auto-continue policy

This plan is executed under `auto_continue` so that the orchestrator does not stop between phases. Each phase must validate, commit, and push before its todo is marked complete. After Phase 8, autonomy ends and the orchestrator reports.
