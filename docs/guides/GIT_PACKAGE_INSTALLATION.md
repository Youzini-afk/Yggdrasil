# Git 安装能力包

> [English](./GIT_PACKAGE_INSTALLATION.en.md) · [中文](./GIT_PACKAGE_INSTALLATION.md)

Yggdrasil host 可以从公开 HTTPS git 仓库安装能力包。它不是 marketplace，也不是包签名网络，更不是让内核变成包管理器。

设计分两层：

- 内核提供受 host policy 控制的 `GitOutboundExecutor`。默认全拒，只支持 HTTPS，不支持 SSH、`git://`、`file://`，不支持私有仓库 token。
- 普通能力包 `official/package-installer-lab` 负责安装计划、审批形状和 profile 级 lockfile。它没有官方特权，第三方可以写替代 installer。

## 当前能做什么

- 新协议方法：`kernel.outbound.git_fetch`。
- 新 manifest 权限：`permissions.git_fetch.hosts`。
- 三类 git executor：
  - `DenyAllGitOutboundExecutor`：默认，全拒；
  - `FakeGitOutboundExecutor`：conformance fixture，不联网；
  - `RealGitOutboundExecutor`：显式 opt-in，调用 host 本机 `git`，公开 HTTPS-only。
- `official/package-installer-lab`：
  - `describe_install_contract`
  - `plan_install`
  - `apply_install`
  - `list_installed`
  - `uninstall`
  - `update`
  - `inspect_lockfile`
- CLI profile lockfile 命令：
  - `ygg package install <git-url> --profile ... --package-id ... --commit-sha ... --content-hash ...`
  - `ygg package list-installed --profile ...`
  - `ygg package uninstall <package-id> --profile ...`
  - `ygg package update <package-id> --profile ... --commit-sha ... --content-hash ...`
  - `ygg package inspect-lockfile --profile ...`

第一轮 CLI 仍要求显式提供 `commit_sha` 与 `content_hash`。真实 git fetch 已经在 executor 层可用；后续会把 CLI 的 install 命令和 `installer-lab` 的 apply 流程接成自动 resolve/pin/apply。

## Profile 配置

Git 安装是 profile 级能力。不开就不能用。

示例见 [`../../profiles/forge-with-git-install.example.yaml`](../../profiles/forge-with-git-install.example.yaml)：

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

重点：

- `enabled` 默认为 `false`。
- `allowed_hosts` 没有通配符默认值。
- `https_only` 必须是 `true`。
- `allow_redirects` 必须是 `false`。
- `executor: real` 才会调用 host 本机 `git`。

## Lockfile

Lockfile 跟 profile 放在一起：

```text
profiles/forge-alpha.yaml
profiles/forge-alpha.lock.yaml
```

结构大致是：

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

Lockfile 只记录 profile 的包集合。它不共享到别的 profile，也不代表跨 host 的全局状态。

## 安全边界

- 只支持公开 HTTPS git 仓库。
- 私有仓库 token、SSH、签名网络、依赖解析、marketplace 都延后。
- URL 不能带用户名、密码或 query token。
- 安装计划必须走审批形状；审批的是具体 commit/content hash，不是一个仓库的未来所有提交。
- 安装阶段不执行 post-install script。
- `official/package-installer-lab` 是普通能力包，不走私有 API。
- YdlTavern 或其他接入项目如果要管理自己的扩展，由它们自己负责；Yggdrasil 只提供通用能力包安装通道。

## 验证

默认 conformance 不联网：

```bash
cargo run -p ygg-cli -- conformance --tag git
```

真实 git fetch 需要显式 opt-in：

```bash
YGG_GIT_INSTALL_REAL_TESTS=1 cargo run -p ygg-cli -- conformance --case git_fetch.real_opt_in
```

默认 CI 不联网。

## 后续

接下来要补的是「自动 resolve/pin/apply」：CLI 和 `installer-lab` 在审批后调用 `kernel.outbound.git_fetch`，自动得到 `commit_sha` 与 `content_hash`，再写 lockfile 并加载包。当前底座已经把 executor、权限、审计、profile lockfile 与官方普通包边界打通。
