# Project Model

> [English](./PROJECT_MODEL.en.md) · [中文](./PROJECT_MODEL.md)

Yggdrasil is a platform. Many projects run on that platform. Each project is like a game on a Steam shelf: an independent entry point, independent state, and something the user can run alone or alongside other projects.

## Three-tier architecture

```text
Kernel (content-free, stable)
  ↓ provides protocol / scheduling / package registration / capability dispatch / event stream / permissions
Capability packages (reusable, shared across projects)
  ↓ provide capabilities (model-provider-lab / persona-lab / ...)
Projects (use capability packages)
  YdlTavern / future coding agent / future image-gen / ...
```

The kernel does not know projects exist. A project is a host/runtime concept, not kernel ontology.

## Steam analogy

| Steam | Yggdrasil |
|---|---|
| Steam client | Yggdrasil platform |
| Game library | Home screen |
| Game card | Project card |
| Game save directory | Per-project data directory |
| Steam wallet | Platform secret store |
| Game-specific DLC key | Project secret store |
| Shared OS drivers | Shared capability packages |

## Project types

Three `project.type` values distinguish where a project came from.

### yggdrasil_native

A repository root has `project.yaml` and references Yggdrasil capability packages. This is the preferred form for projects designed for Yggdrasil.

```yaml
schema_version: 1
project:
  id: my-project__abc12345
  title: My Project
  description: A short summary
  type: yggdrasil_native
  entry_surface_id: my-namespace/play
  packages:
    - packages/foo/manifest.yaml
    - packages/bar/manifest.yaml
  secret_policy:
    fallback_to_platform: true
```

### external_wrapped

An external project, such as an ordinary git or npm repository, wrapped by an adapter package. If install chooses "wrap with adapter", Yggdrasil uses `adapter-generator-lab` to generate an adapter package and connect the external project.

### external_workspace

An external project connected as an agent workspace, without wrapping. This fits temporary use and agent-assisted modification. It is the default when there is no TTY and no explicit flag.

## ProjectDescriptor

The top level of `project.yaml` is a `ProjectDescriptor`. It describes a project instance, not one package by itself.

Common fields:

| Field | Meaning |
|---|---|
| `id` | Stable project id for directories, CLI, and Home cards. |
| `title` | User-facing project name. |
| `description` | Text shown on Home cards and detail views. |
| `type` | `yggdrasil_native` / `external_wrapped` / `external_workspace`. |
| `entry_surface_id` | Surface contribution id opened by Play. |
| `packages` | Required package manifest paths. |
| `optional_packages` | Optional package manifest paths. |
| `required_surfaces` | Surface ids the project expects to exist. |
| `secret_policy` | Project secret resolution policy. |

`entry_surface_id` should match a package manifest surface with `slot: experience_entry`.
For example, YdlTavern uses `ydltavern/play`.

## Project directory layout

```text
~/.yggdrasil/projects/<project_id>/
├── project.yaml          # ProjectDescriptor copy
├── secrets.dat           # age-encrypted project secret store
├── sessions/             # project-level session data
├── state/                # project-level state packages may use
└── lockfile.toml         # package versions locked for this project
```

Permissions: 0700 directories, 0600 files on Unix.
Encryption: the same master key, from `~/.yggdrasil/secret-store.key` or the OS keyring.

## Soft isolation + platform fallback

Project isolation is soft, not tenant-grade hard isolation. Default behavior:

- The project's own secret wins (`secret_ref:project:NAME`).
- If missing in the project, fall back to platform when `secret_policy.fallback_to_platform: true`.
- If missing in both, fail closed.

The intent: a user can configure `OPENAI_API_KEY` once at the platform level and all projects can use it. A specific project can override with a project-level key in that project's settings. Both paths are visible to the user; fallback is not hidden.

Strong-isolation projects can disable fallback:

```yaml
secret_policy:
  fallback_to_platform: false
  require_per_project:
    - GITHUB_PAT       # must be configured per-project; platform fallback is not allowed
```

## Lifecycle

```text
yg install <url>
  ↓ (detect project.yaml / run wizard)
Installed (registered in ProjectRegistry, visible in Home)
  ↓ yg project start (or Home Play)
Starting → Running
  ↓ yg project stop
Stopping → Stopped
  ↓ yg uninstall
(ask what to do with data)
  ├─ Keep: move to ~/.yggdrasil/projects/.archived/<id>/
  └─ Delete: rm -rf immediately
```

Any state can fail → Failed.

## CLI commands

```bash
# Install projects
yg install github.com/user/repo
yg install github.com/user/repo --wrap-as-adapter   # external project: wrap
yg install github.com/user/repo --workspace-only    # external project: workspace

# Inspect projects
yg project list
yg project info <id>
yg project status <id>

# Control
yg project start <id>
yg project stop <id>

# Uninstall
yg uninstall <id>                # interactive data prompt
yg uninstall <id> --keep-data    # keep data (move to .archived)
yg uninstall <id> --delete-data  # delete immediately
```

## Home screen

The `clients/web` Home route shows cards for all installed projects:

```text
┌─────────────────┐  ┌─────────────────┐
│   YdlTavern     │  │  Coding Agent   │
│   ●Running      │  │  ◯Stopped       │
│   [Play]        │  │  [Play]         │
└─────────────────┘  └─────────────────┘
┌─────────────────┐
│  + Install      │
└─────────────────┘
```

Status indicators:

- ● Running (green)
- ◯ Stopped / Installed (gray)
- ⏳ Starting / Stopping (yellow)
- ❌ Failed (red)

Clicking Play calls `kernel.v1.project.start`, then navigates to the project's `entry_surface`.

## Play flow

After a user clicks Play on a Home card, the web shell and host follow a fixed public-protocol sequence:

1. The user clicks Play on a project card.
2. `clients/web` calls `kernel.v1.project.start`.
3. The host transitions the project to Running and creates or reuses a project session.
4. The session stores `metadata.project_id` and gets a `project:<id>` label.
5. `project.start` returns `session_id` and `already_running`.
6. `clients/web` calls `kernel.v1.surface.resolve_bundle` to resolve the project's `entry_surface_id` to a surface bundle URL.
7. `mountSurface` mounts a sandboxed iframe.
8. The iframe `initialProps` include `sessionId` and `projectId`.
9. Inside the surface, `callHostRpc` / `invokeCapability` automatically carries `session_id`.
10. The host carries `ProtocolContext.session_id` into later capability and outbound dispatch.

This chain lets project-level secret resolution find the project scope from session metadata, and it keeps real model calls in the same project session. For the end-to-end path, see [`REAL_MODEL_END_TO_END.md`](REAL_MODEL_END_TO_END.en.md).

Note: this `sessionId` is then used for:

- All RPC calls, which carry it automatically (`callHostRpc` reads it through `setActiveSessionId`).
- Streaming calls (`streamCapability`), which use it as the subscription scope for receiving `kernel/v1/stream.*` events.

## Protocol

Host project-management methods are HostAdmin/HostDev only; ordinary packages cannot call them:

```text
kernel.v1.project.list      list installed projects
kernel.v1.project.get       get project details
kernel.v1.project.start     start a project
kernel.v1.project.stop      stop a project
kernel.v1.project.status    get project status
```

Lifecycle events:

```text
kernel/v1/project.installed
kernel/v1/project.started
kernel/v1/project.stopped
kernel/v1/project.uninstalled
```

## Difference from Composition

| Composition (existing) | Project (new) |
|---|---|
| Static package-set descriptor | Runtime instance + state |
| Validated by `ygg composition check` | Managed by `yg project list/start/stop` |
| Used for share/import bundles | Used for Home + install lifecycle |
| `ygg-cli` internal type | `ygg-core` public type |

In the future, one composition template can instantiate multiple projects with different ids and the same package set. The current version does not require that; one project is usually one concrete composition instance.

## Install detection

`yg install <url>` first checks the repository root for `project.yaml`.

- Present with `type: yggdrasil_native`: install as a native project.
- Absent: enter the external-project wizard.
- Present but invalid: fail closed and require descriptor fixes.

Without a TTY and without explicit flags, an external project defaults to `external_workspace` to avoid generating wrapper code implicitly.

## Non-goals (deferred)

- Multi-user project membership / access control
- Project import/export bundles (sharing-lab already has bundle formats)
- Concurrent multi-tenancy (`project_id` in `ProtocolContext`) — deferred / planned
- Automatic project archive cleanup (manual for now)
- Project marketplace (against the platform's open principle)
