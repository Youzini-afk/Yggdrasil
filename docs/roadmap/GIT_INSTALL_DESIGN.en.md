# Git install design

> [English](./GIT_INSTALL_DESIGN.en.md) · [中文](./GIT_INSTALL_DESIGN.md)

Letting a Yggdrasil host install capability packages from public HTTPS git repos. This document fixes the architecture, contracts, security boundary, and rollout order.

## Stance

The platform needs a controlled path for pulling in external code — but that path must not teach the kernel how to "install packages."

The design splits in two:

- **Kernel layer** provides a generic, host-policy-controlled git outbound channel. It mirrors `LiveHttpOutboundExecutor`: deny-all by default, host opts in explicitly, HTTPS-only, every access goes through audit and redaction.
- **Capability-package layer** carries the actual install logic in an ordinary official package, `official/package-installer-lab`: parsing manifests, drafting proposals, writing the lockfile, registering new packages.

The kernel doesn't know what "git repo," "package install," or "dependency resolution" mean. It only knows there's a controlled outbound shape called "git fetch" that capability packages can use — same posture as `kernel.outbound.execute`.

## Architecture

```text
┌───────────────────────────────────────────────────────────┐
│  CLI / Web shell / third-party clients                     │
│  · ygg package install <github-url>                        │
└───────────────────────────────────────────────────────────┘
                            │ public protocol
                            ▼
┌───────────────────────────────────────────────────────────┐
│  official/package-installer-lab (ordinary capability pkg)  │
│  · plan_install      drafts proposal, requires approval    │
│  · apply_install     real fetch + register only after OK   │
│  · list_installed                                           │
│  · uninstall                                                │
│  · update                                                   │
│  · inspect_lockfile                                         │
└───────────────────────────────────────────────────────────┘
                            │ kernel.outbound.git_fetch
                            ▼
┌───────────────────────────────────────────────────────────┐
│  Yggdrasil kernel                                           │
│  · GitOutboundExecutor trait (DenyAll by default)          │
│  · Config: host enable, HTTPS-only, destination allowlist  │
│  · Audit + redaction + proposal-aware                       │
└───────────────────────────────────────────────────────────┘
                            │ HTTPS
                            ▼
                    public git repos
```

## Kernel: `GitOutboundExecutor`

A separate trait, parallel to `OutboundExecutor`. Reason: a git fetch is a repo-plus-ref operation, not a single request/response — forcing it into HTTP-shaped audit would be awkward.

### Request and response (draft)

```text
GitOutboundRequest
  package_id            calling capability package
  capability_id         calling capability
  remote_url            HTTPS git URL
  ref                   branch / tag / commit SHA
  fetch_kind            shallow_clone | tree_only | refs_only
  destination_hint      caller's preferred install location (host may override)
  secret_refs           optional: token for private repo access, secret_ref form
  redaction_state       redacted by default

GitOutboundResponse
  status                ok | denied | error | timeout
  resolved_commit_sha   the real commit SHA the ref resolved to
  resolved_content_hash hash over the full tree (FNV1a64 or SHA-256)
  resolved_path         install root subdir picked by the host
  redaction_state       redacted
  network_performed     true | false
  executor_kind         deny_all | fake | real
```

Neither side carries raw tokens, raw query-token URLs, or git-protocol verbose details beyond the ref name.

### Three implementations

- `DenyAllGitOutboundExecutor`: default. Every call returns `denied`.
- `FakeGitOutboundExecutor`: for conformance. The host holds a set of fixture repo contents indexed by `(remote_url, ref)`. No real network.
- `RealGitOutboundExecutor`: opt-in. Built on the [`gix`](https://github.com/Byron/gitoxide) crate (recommended). HTTPS-only, shallow clones, SSH and `file://` rejected, no redirects to non-HTTPS.

### Host policy fields

New section in profile YAML:

```yaml
outbound:
  git:
    enabled: false                         # off by default
    executor: deny_all                     # deny_all | fake | real
    allowed_hosts:                         # required when real, no wildcard default
      - github.com
      - gitlab.com
      - codeberg.org
    https_only: true                       # forced
    max_clone_size_mb: 64
    timeout_ms: 30000
    install_root: ~/.local/share/ygg/installed-packages
    allow_redirects: false
```

### Protocol method

One new method:

```text
kernel.outbound.git_fetch
```

Called by capability packages. Sits parallel to `kernel.outbound.execute`. Governed by outbound policy; a package must declare `permissions.git_fetch.hosts` in its manifest (same shape as `permissions.network.hosts`).

We do **not** add `kernel.package.install_*`, `kernel.git.*`, `kernel.repository.*`, or `kernel.dependency.*` namespaces. Those are package concerns.

### Audit events

```text
kernel/git_fetch.requested
kernel/git_fetch.denied
kernel/git_fetch.completed
kernel/git_fetch.failed
```

Payloads carry `package_id`, `capability_id`, `remote_url`, `ref`, `resolved_commit_sha`, `resolved_content_hash`, `status`, `redaction_state`. Raw tokens, raw query strings, and git-protocol verbose output never enter events.

## Capability package: `official/package-installer-lab`

An ordinary capability package — same rules as any other official package. Same manifest, same permission gate, replaceable by any third-party package.

### Capabilities (draft)

```text
describe_install_contract
  Describes the install contract: supported ref forms, required permissions,
  proposal shape, lockfile shape.

plan_install
  Input:  remote_url, ref (defaults to main), preferred_package_id (optional)
  Output: proposal_draft with resolved commit_sha, manifest preview,
          declared permissions, estimated fetch size, install_root subdir,
          requires_user_approval=true
  This step only fetches refs/manifest blob — no full tree clone, no register.

apply_install
  Input:  approved proposal_id
  Action: full fetch (bounded by fetch_kind / size cap) → write to install_root
          → validate manifest → call kernel.package.load → write lockfile entry
  Cleans up landed files on failure — no half-installed state.

list_installed
  Reads lockfile and the host install dir; outputs package_id, remote_url,
  commit_sha, content_hash, installed_at.

uninstall
  Input:  package_id
  Action: kernel.package.unload → remove install_root subdir → remove lockfile entry
  Emits audit events; cannot bypass the proposal flow.

update
  Input:  package_id, target_ref (defaults to repo default branch)
  Action: an update-shaped plan_install — diffs against current lockfile
          commit_sha and drafts an "X→Y" proposal; apply unloads old, installs new.

inspect_lockfile
  Describes the current profile's lockfile state, dangling packages, mismatched entries.
```

These are ordinary capabilities. The package can call `kernel.outbound.git_fetch` because its own manifest declares `permissions.git_fetch.hosts` and the host policy allows it.

### Proposal shape

`apply_install` is not a direct action. The proposal_draft from `plan_install` runs through the standard `kernel.proposal.*` flow:

```text
proposal_draft:
  operations:
    - kind: package.install
      remote_url: https://github.com/...
      ref: main
      resolved_commit_sha: <sha>
      resolved_content_hash: <hash>
      manifest_preview: { … }
      requested_permissions: { … }
      install_root_subdir: <path>
  expected_effects:
    - registers new capability package package_id=...
    - uses ~12.4 MB of disk
    - requests permissions network.hosts=[…], filesystem_read=[…]
  requires_user_approval: true
  source_ref: official/package-installer-lab/plan_install
```

No approval, no install. Approval covers "this commit_sha plus this permission set" — not "every future commit in this repo."

### Trust model

```text
Pin    commit SHA           — required, written to events and lockfile
Pin    full-tree content hash — required, secondary check on commit SHA
Pin    manifest content address — required, plan and apply must match
Sign   git tag GPG / SSH signatures — not in the first round
```

The first round skips package signing networks, dependency resolution, and dependency graphs — all deferred alongside the marketplace question.

## Lockfile

Profile-scoped, not host-scoped.

Each profile maintains its own lockfile, sitting next to the profile YAML:

```text
<profile-yaml-dir>/<profile-name>.lock.yaml
```

For example, `profiles/forge-alpha.yaml` pairs with `profiles/forge-alpha.lock.yaml`.

### Lockfile shape (draft)

```yaml
format_version: 1
profile: forge-alpha
generated_at: 2026-05-21T12:00:00Z

packages:
  - package_id: thirdparty/some-package
    remote_url: https://github.com/example/some-package
    ref: main
    commit_sha: abcd1234...
    content_hash: fnv1a64:...
    manifest_path: manifest.yaml
    installed_at: 2026-05-20T18:30:00Z
    install_root_subdir: thirdparty-some-package-abcd1234

forbidden_overrides:
  - Do not hand-edit commit_sha or content_hash;
  - These fields are written and maintained by installer-lab.
```

Why profile-scoped:

- Each host configuration (`forge-alpha`, `forge-postgres-example`, `tavern-host`, etc.) keeps its own package set; nothing shared.
- Switching profile = switching package set, with no leftover packages from the previous one.
- Profile is already a host-config concept — reusing it doesn't add anything new.

If hosts want to share an actual on-disk cache, point multiple profiles' `install_root` at the same directory. The lockfiles still stay independent.

## CLI

```bash
ygg package install <github-url> [--ref <branch|tag|sha>] [--profile <name>]
ygg package list-installed [--profile <name>]
ygg package uninstall <package_id> [--profile <name>]
ygg package update <package_id> [--ref <branch|tag|sha>] [--profile <name>]
ygg package inspect-lockfile [--profile <name>]
```

The CLI is a thin client of `installer-lab`. It calls capabilities via the public protocol — no privilege.

## Security boundary

Red lines that hold across every phase:

- **HTTPS-only.** Any SSH, `git://`, or `file://` is rejected outright.
- **Destination allowlist required.** No `*` wildcard default.
- **Deny-all default.** Without `outbound.git.enabled: true` in the profile, no install can happen.
- **Approval flow cannot be bypassed.** `apply_install` only takes host-side approved `proposal_id`s. There is no auto-approve CLI flag.
- **Plan/apply consistency.** Apply re-resolves `commit_sha` and `content_hash`; mismatch with the proposal aborts.
- **Tokens via `secret_ref` only.** Private repos are deferred; when supported, tokens go through `secret_ref:env:NAME` with explicit host allowlist.
- **Disk cap.** Anything over `max_clone_size_mb` aborts.
- **Redirects fail closed.** Same as `LiveHttpOutboundExecutor`.
- **Everything audited.** Every fetch, deny, complete, fail goes to the event log with `redaction_state=redacted`.
- **No post-install scripts.** The first round forbids any in-package hook from running arbitrary code at install time. Capability packages only get to run code after `kernel.package.load` — same rule as everyone else.

## Conformance plan (draft)

First-round cases:

```text
git_fetch.deny_all_default                   default profile rejects all git_fetch
git_fetch.requires_https                     http:// / git@ / file:// all rejected
git_fetch.requires_host_allowlist            unlisted host is rejected
git_fetch.fake_executor_returns_fixture      fake executor fixture roundtrip
git_fetch.audit_no_raw_secrets               audit events carry no raw token

installer_lab.contract_shape                 contract shape
installer_lab.plan_install_no_apply          plan stage doesn't write to disk
installer_lab.apply_install_requires_proposal apply without approval fails
installer_lab.plan_apply_consistency         commit_sha mismatch aborts
installer_lab.lockfile_round_trip            install, read lockfile, uninstall, re-read
installer_lab.update_diff_preview            update drafts an X→Y proposal
installer_lab.uninstall_cleans_disk          no leftover after uninstall
installer_lab.no_kernel_namespace_leak       no kernel.git/repository/dependency in output
```

Real-network conformance is opt-in via env var (mirroring `YGG_TDB_REAL_TESTS=1`):

```text
YGG_GIT_INSTALL_REAL_TESTS=1
  runs a small set of real fetches against github.com/Youzini-afk/<fixture repo>
```

Default CI never goes online.

## Implementation order

No Alpha/Beta/Phase naming — just steps, each independently doable, committable, and verifiable:

1. Kernel: `GitOutboundRequest` / `GitOutboundResponse` types, `GitOutboundExecutor` trait, `DenyAllGitOutboundExecutor`, unit tests for the deny-all default.
2. Kernel: profile parses `outbound.git`, new audit event kinds, red-line unit tests (HTTPS-only, allowlist required, deny-all default).
3. Kernel: `FakeGitOutboundExecutor` plus a fixture-repo set (no network); `kernel.outbound.git_fetch` protocol method; conformance covering the first five cases above.
4. Package: `official/package-installer-lab` skeleton, `describe_install_contract` plus `plan_install` (only the manifest blob, no full clone), proposal flow end-to-end.
5. Package: `apply_install` plus lockfile writes, calls into `kernel.package.load`, conformance running the install loop over the fake executor.
6. Package: `list_installed`, `uninstall`, `inspect_lockfile`, `update`, with matching conformance.
7. Kernel: `RealGitOutboundExecutor` (on `gix`), HTTPS-only, size cap, timeout, redirect fail-closed; opt-in real fetch case under `YGG_GIT_INSTALL_REAL_TESTS=1`.
8. CLI commands, doc convergence, an example `profiles/forge-with-git-install.example.yaml`.

Each step ships and pushes before the next one starts.

## Relationship to YdlTavern

YdlTavern manages its extensions itself. If it needs to install Yggdrasil capability packages from git to support its own features, it can call `installer-lab` — but Yggdrasil sees that as an ordinary package install, not anything Tavern-specific.

Yggdrasil never adds a special path for YdlTavern's extension ecosystem.

## Out of scope (first round)

Off the table for this round, to keep the design from being pulled around by product needs:

- **Private repos / token auth.** Stabilize the public HTTPS path first.
- **SSH transport.** Use HTTPS with a deploy key or token instead.
- **Package signing / trust networks.** `commit_sha` plus `content_hash` covers integrity.
- **Dependency resolution.** One git URL, one capability package. No recursive deps.
- **Marketplace / registry.** A git URL is the whole identity.
- **Post-install scripts.** Never run arbitrary code during install.
- **Cross-host shared lockfiles.** Profile-scoped only.

## Red lines

- No `kernel.git.*` / `kernel.repository.*` / `kernel.package.install_*` / `kernel.dependency.*` namespaces.
- No git-library domain types (commit, tree, refspec) leaking into the kernel — those stay inside the capability package.
- The official `installer-lab` gets no privilege — same permission gate, same audit, same approval flow. Any third party can ship an equivalent installer package; if host policy allows it, it works.
- The install path doesn't bind to a specific git library. `gix` is the current pick, but the trait stays decoupled from any crate.

## Next

Once this lands, work starts on step 1. The whole effort doesn't sit under an "Alpha + Beta + Phase" name — it's a piece of background substrate work, finished when it's finished.
