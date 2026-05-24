# Round 11 — Host Integration (Real install pipeline / real crash capture / disk usage)

> [English](./ROUND_11_HOST_INTEGRATION.en.md) · [中文](./ROUND_11_HOST_INTEGRATION.md)

After the platform shell rebuild, the Web shell still mocks three things on the
client side: install progress prototype, failure-modal demo defaults, and zero-
byte disk usage. Round 11 wires those three to the real host/kernel pipeline,
all through public protocol, all plan-first, and without introducing any
content-shaped `kernel.v1.install/crash/disk` ontology.

## Overall principles

```
✗ no kernel.v1.install.*       (install is an ordinary capability package)
✗ no kernel.v1.crash.*         (failure is project lifecycle → project.failed)
✗ no kernel.v1.disk.*          (disk is a project summary → project list/get/status field)

✓ install-lab emits package-namespaced progress events on existing protocol (official/install-lab/install.*)
✓ project failure uses kernel/v1/project.failed (existing project.* namespace)
✓ storage_summary lives on ProjectRecord, returned by project list/get/status
```

## Phase A — Real install pipeline

### Problem

* `official/install-lab` already implements the full set
  (`resolve_plan / execute_plan / detect_kind / register_project / uninstall /
  list_installed / check_lockfile`), but only the `yg install` CLI calls it.
* Web `InstallModal`'s 3-step flow is a pure front-end prototype: URL input →
  fake plan → simulated progress.
* No progress events, so Web cannot show real "cloning X / verifying Y / writing
  to store" steps.

### Solution

1. Add `append_event` to the `InprocCapabilityInvoker` trait, symmetrical to
   the existing `invoke_capability` / `project_registry`. This gives every
   inproc package a unified reverse "write event" channel that carries
   principal, runs through schema validation, and never bypasses permissions.
2. install-lab's `resolve_plan` / `execute_plan` emit package-namespaced events
   at key points:
   * `official/install-lab/install.plan.resolving`
   * `official/install-lab/install.plan.resolved` (package count / permissions /
     signatures summary)
   * `official/install-lab/install.execute.started`
   * `official/install-lab/install.execute.package.fetching`
   * `official/install-lab/install.execute.package.fetched`
   * `official/install-lab/install.execute.package.verified`
   * `official/install-lab/install.execute.completed`
   * `official/install-lab/install.execute.failed`
3. Write package-owned JSON Schemas / manifests for these payloads under
   `docs/spec/v1/schemas/event/official.install-lab.*.schema.json`; do not
   register them in the kernel `EVENT_KIND_REGISTRY`.
4. Rework Web `InstallModal`:
   * Step 1 submits URL → opens session (`kernel.v1.session.open`) → invokes
     `official/install-lab/resolve_plan` → renders real package count /
     permissions / signature summary.
   * Step 2 user reviews, hits Install → invokes
     `official/install-lab/execute_plan` while subscribing to the session's
     `official/install-lab/install.*` event stream over SSE.
   * Step 3 progress is driven by real events (clone X / verify Y / wrote
     lockfile).
   * Failure / cancel branches off real events.
5. New conformance cases:
   * `install_lab_emits_progress_events`
   * `install_lab_failure_emits_failed_event`

### Out of scope

* No `kernel.v1.install.*` protocol method.
* No change to install-lab's CLI behavior (backward compatible).
* No GPG signature smoke (existing `--require-signed` flag covers it).

## Phase B — Real crash capture

### Problem

* `SubprocessSupervisor` buffers stderr but with no ring-buffer ceiling, and on
  child death the supervisor only sees a broken reverse pump or an invoke
  error.
* No `kernel/v1/project.failed` event, and project state does not flip to
  Failed on crash.
* `ProjectRegistry` has no `last_failure` field.
* Web `FailureModal` currently shows hardcoded demo defaults (exit 137 / OOM /
  fake logs).

### Solution

1. Add a stderr ring buffer to `SubprocessHandle`:
   * 64 KB ceiling (configurable, defaults to 64 KB)
   * line-based, drops oldest whole line on overflow
   * `drain_recent_stderr() -> Vec<String>`
2. Supervisor watches for child exit:
   * On non-zero / signal, capture `exit_code: Option<i32>`,
     `signal: Option<i32>`, `stderr_tail: Vec<String>`, `duration_ms`.
   * Reverse-write `kernel/v1/project.failed` via
     `InprocCapabilityInvoker.append_event` if the dying package is bound to a
     project.
3. Add `last_failure: Option<ProjectFailure>` to `ProjectRegistry`:
   ```rust
   struct ProjectFailure {
       at: DateTime<Utc>,
       exit_code: Option<i32>,
       signal: Option<i32>,
       stderr_tail: Vec<String>,
       duration_ms: u64,
       package_id: PackageId,
   }
   ```
4. `kernel.v1.project.list` returns `last_failure: Option<ProjectFailureSummary>`
   on each project (redacted: `stderr_tail` only for `host_admin` / `host_dev`).
5. New `kernel.v1.project.failure` method (host_admin / host_dev only) returns
   the full failure detail.
6. Rework Web `FailureModal`:
   * Reads real `exit_code` / `signal` / `stderr_tail` from
     `kernel.v1.project.failure`.
   * No more hardcoded 137 / OOM.
   * Shows a real empty state when no failure is recorded.
7. New conformance cases:
   * `subprocess_crash_emits_project_failed_event`
   * `project_failure_method_redacts_stderr_for_anonymous`
   * `project_failure_method_returns_full_data_for_host_admin`

### Out of scope

* No automatic restart logic (user decides).
* No persisted crash history (only last failure; full history lives in the
  event log via `list_events`).

## Phase C — Project storage-summary disk usage

### Problem

* Project list/get/status have no `storage_summary` field.
* Web Disk Usage always shows zero bytes.

### Solution

1. Add `storage_summary` to runtime project list/get/status results. It only
   includes byte counts, `measured_at`, and `measurement_state`; it never exposes
   host paths or filesystem trees.
2. Measure `ygg_core::paths::project_dir(id)` by recursively summing file sizes
   without following symlinks; read failures return `unknown` / `null`.
3. Web `WorkshopUtilities`'s `DiskSegment.bytes` reads
   `ProjectRecord.storage_summary.total_bytes`.
4. New conformance case `project_record_includes_storage_summary`.

### Out of scope

* No disk quota / alerts.
* No background disk monitor (lazy compute + cache only).

## Order

Each Phase commits and pushes independently. Final consolidated report at the
end.

```
A → B → C
```

A introduces `InprocCapabilityInvoker.append_event`, which Phase B also needs
(supervisor reverse-writes events). C is independent.

## Doc convergence (final)

* Delete `docs/roadmap/ROUND_11_HOST_INTEGRATION.{md,en.md}` once all phases
  land.
* Update `docs/ALPHA_STATUS.{md,en.md}` Web shell + project + install sections.
* Update `docs/roadmap/NEXT_STEPS.{md,en.md}` to move these three items from
  "deferred" to "done".
* Update install-lab package-owned event schema / manifest docs.
* Update `clients/web/README.md` Install / Failure / Storage data wiring notes.
