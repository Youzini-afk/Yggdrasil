# Git package installation

> [English](./GIT_PACKAGE_INSTALLATION.en.md) · [中文](./GIT_PACKAGE_INSTALLATION.md)

A Yggdrasil host can install capability packages from public HTTPS git repositories. This is not a marketplace, not a package-signing network, and not a package manager baked into the kernel.

The design has two layers:

- The kernel provides `GitOutboundExecutor`, controlled by host policy. It denies by default, supports HTTPS only, rejects SSH / `git://` / `file://`, and does not support private-repo tokens yet.
- The ordinary capability package `official/package-installer-lab` owns install plans, approval shape, and the profile-scoped lockfile. It has no official privilege; a third-party installer can replace it.

## What works now

- New protocol method: `kernel.outbound.git_fetch`.
- New manifest permission: `permissions.git_fetch.hosts`.
- Three git executors:
  - `DenyAllGitOutboundExecutor`: default, rejects everything;
  - `FakeGitOutboundExecutor`: conformance fixture, no network;
  - `RealGitOutboundExecutor`: explicit opt-in, calls the host `git` binary, public HTTPS only.
- `official/package-installer-lab`:
  - `describe_install_contract`
  - `plan_install`
  - `apply_install`
  - `list_installed`
  - `uninstall`
  - `update`
  - `inspect_lockfile`
- CLI profile lockfile commands:
  - `ygg package install <git-url> --profile ... --package-id ... --commit-sha ... --content-hash ...`
  - `ygg package list-installed --profile ...`
  - `ygg package uninstall <package-id> --profile ...`
  - `ygg package update <package-id> --profile ... --commit-sha ... --content-hash ...`
  - `ygg package inspect-lockfile --profile ...`

The first CLI slice still requires explicit `commit_sha` and `content_hash`. Real git fetch exists at the executor layer; the next step is wiring CLI install and `installer-lab` apply into automatic resolve/pin/apply.

## Profile configuration

Git install is profile-scoped. If the profile doesn't enable it, it can't run.

Example: [`../../profiles/forge-with-git-install.example.yaml`](../../profiles/forge-with-git-install.example.yaml):

```yaml
outbound:
  git:
    enabled: true
    executor: real
    allowed_hosts:
      - github.com
      - gitlab.com
      - codeberg.org
    https_only: true
    max_clone_size_mb: 64
    timeout_ms: 30000
    install_root: ./.ygg-installed-packages
    allow_redirects: false
```

Key points:

- `enabled` defaults to `false`.
- `allowed_hosts` has no wildcard default.
- `https_only` must be `true`.
- `allow_redirects` must be `false`.
- `executor: real` is what calls the host `git` binary.

## Lockfile

The lockfile sits next to the profile:

```text
profiles/forge-alpha.yaml
profiles/forge-alpha.lock.yaml
```

Shape:

```yaml
format_version: 1
profile: forge-alpha
generated_at: unix:1760000000
packages:
  - package_id: thirdparty/example
    remote_url: https://github.com/example/ygg-package
    ref: main
    commit_sha: 0123456789abcdef0123456789abcdef01234567
    content_hash: sha256:...
    manifest_path: manifest.yaml
    installed_at: unix:1760000000
    install_root_subdir: thirdparty-example-0123456789ab
```

The lockfile records only that profile's package set. It is not shared with other profiles and does not describe global state across hosts.

## Security boundary

- Public HTTPS git repos only.
- Private-repo tokens, SSH, signing networks, dependency resolution, and marketplace behavior are deferred.
- URLs may not include usernames, passwords, or query tokens.
- Install plans use approval shape; approval covers one pinned commit/content hash, not future commits in a repo.
- No post-install scripts.
- `official/package-installer-lab` is an ordinary package and does not use private APIs.
- YdlTavern and other integration projects manage their own extension systems. Yggdrasil only provides the generic capability-package install path.

## Verification

Default conformance does not go online:

```bash
cargo run -p ygg-cli -- conformance --tag git
```

Real git fetch is opt-in:

```bash
YGG_GIT_INSTALL_REAL_TESTS=1 cargo run -p ygg-cli -- conformance --case git_fetch.real_opt_in
```

Default CI never performs network fetches.

## Next

The remaining integration is automatic resolve/pin/apply: after approval, CLI and `installer-lab` call `kernel.outbound.git_fetch`, get `commit_sha` and `content_hash`, write the lockfile, and load the package. The substrate now has the executor, permissions, audit, profile lockfile, and ordinary official package boundary in place.
