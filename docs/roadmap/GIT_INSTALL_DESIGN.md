# Git 安装能力设计

> [English](./GIT_INSTALL_DESIGN.en.md) · [中文](./GIT_INSTALL_DESIGN.md)

让 Yggdrasil host 能从公开 HTTPS git 仓库地址安装能力包。这份文档固定架构、契约、安全边界与分阶段实施策略。

## 立场

平台需要一条受控的外部代码引入路径——但这条路径不能让内核学会「装包」。

设计采用两层：

- **内核层** 提供一个通用的、受 host policy 管控的 git 出站通道。它像 `LiveHttpOutboundExecutor` 一样：默认全拒，host 显式启用，HTTPS-only，所有访问走审计与脱敏。
- **能力包层** 由普通官方能力包 `official/package-installer-lab` 承担实际的安装逻辑：解析 manifest、生成提案、写 lockfile、注册新包。

内核不知道「git 仓库」「能力包安装」「依赖解析」是什么。它只知道有一类受控的出站请求叫「git fetch」，能力包用它，跟用 `kernel.outbound.execute` 没本质区别。

## 架构

```text
┌───────────────────────────────────────────────────────────┐
│  CLI / Web shell / 第三方客户端                              │
│  · ygg package install <github-url>                        │
└───────────────────────────────────────────────────────────┘
                            │ 公开协议
                            ▼
┌───────────────────────────────────────────────────────────┐
│  official/package-installer-lab（普通能力包）                  │
│  · plan_install      生成提案，要求审批                       │
│  · apply_install     审批后才真正 fetch + register             │
│  · list_installed                                           │
│  · uninstall                                                │
│  · update                                                   │
│  · inspect_lockfile                                         │
└───────────────────────────────────────────────────────────┘
                            │ kernel.outbound.git_fetch
                            ▼
┌───────────────────────────────────────────────────────────┐
│  Yggdrasil 内核                                              │
│  · GitOutboundExecutor trait（默认 DenyAll）                 │
│  · 配置：host 是否启用、HTTPS-only、目标 host allowlist        │
│  · 把请求走审计 + 脱敏 + 提案路径                                │
└───────────────────────────────────────────────────────────┘
                            │ HTTPS
                            ▼
                    公开 git 仓库
```

## 内核：`GitOutboundExecutor`

跟现有 `OutboundExecutor` 平行，单独的 trait——理由：git fetch 是仓库 + ref 操作，不是单次 request/response，硬塞进 HTTP-shaped 审计不自然。

### 请求与响应（草案）

```text
GitOutboundRequest
  package_id            发起请求的能力包
  capability_id         发起请求的能力
  remote_url            HTTPS git 仓库地址
  ref                   branch / tag / commit SHA
  fetch_kind            shallow_clone | tree_only | refs_only
  destination_hint      请求方期望的 host 内部安装位置（host 可拒）
  secret_refs           可选：访问私有仓库时用的 token，secret_ref 形式
  redaction_state       默认 redacted

GitOutboundResponse
  status                ok | denied | error | timeout
  resolved_commit_sha   ref 解析后的真实 commit SHA
  resolved_content_hash 整树 hash（FNV1a64 或 SHA-256）
  resolved_path         host 选定的安装根目录
  redaction_state       redacted
  network_performed     true | false
  executor_kind         deny_all | fake | real
```

请求 / 响应里都不出现原始 token、原始 URL 中的 query token、原始 ref name 之外的 git 协议细节。

### 三档实现

- `DenyAllGitOutboundExecutor`：默认。任何调用直接返回 `denied`。
- `FakeGitOutboundExecutor`：conformance 用。host 持有一组 fixture 仓库内容，按 `(remote_url, ref)` 返回。不走真实网络。
- `RealGitOutboundExecutor`：opt-in。基于 [`gitoxide`](https://github.com/Byron/gitoxide) 的 `gix` crate（推荐），HTTPS-only，shallow clone，禁用 SSH 与 file://，不跟随重定向到非 HTTPS。

### Host policy 字段

profile YAML 里新增：

```yaml
outbound:
  git:
    enabled: false                         # 默认全关
    executor: deny_all                     # deny_all | fake | real
    allowed_hosts:                         # 真实启用时必填，无通配符默认值
      - github.com
      - gitlab.com
      - codeberg.org
    https_only: true                       # 强制
    max_clone_size_mb: 64
    timeout_ms: 30000
    install_root: ~/.local/share/ygg/installed-packages
    allow_redirects: false
```

### 协议方法

新增一条：

```text
kernel.outbound.git_fetch
```

由能力包调用，跟 `kernel.outbound.execute` 是平行关系。受 outbound policy 管控；package 必须在 manifest 里声明 `permissions.git_fetch.hosts`（与 `permissions.network.hosts` 同样的形态）。

不新增 `kernel.package.install_*`、`kernel.git.*`、`kernel.repository.*`、`kernel.dependency.*` 这些 namespace。这些都是能力包的事。

### 审计事件

```text
kernel/git_fetch.requested
kernel/git_fetch.denied
kernel/git_fetch.completed
kernel/git_fetch.failed
```

事件 payload 含 `package_id`、`capability_id`、`remote_url`、`ref`、`resolved_commit_sha`、`resolved_content_hash`、`status`、`redaction_state`。原始 token、原始 query string、git protocol verbose 输出都不进事件。

## 能力包：`official/package-installer-lab`

普通能力包，跟其他官方包同样规则——同一份 manifest，同一道权限闸门，可被第三方包替换。

### 能力清单（草案）

```text
describe_install_contract
  描述安装契约：支持的 ref 形式、所需权限、提案形态、lockfile 形态。

plan_install
  输入：remote_url, ref（默认 main），preferred_package_id（可选）
  输出：proposal_draft，含解析到的 commit_sha、manifest 预览、声明的权限、
       预估 fetch 大小、所属 install_root 子目录、requires_user_approval=true
  这一步只 fetch refs / manifest blob，不 clone full tree，不注册包。

apply_install
  输入：被审批的 proposal_id
  执行：full fetch（受 fetch_kind / size cap 限制）→ 写入 install_root →
       校验 manifest 合法性 → 调用 kernel.package.load → 写 lockfile 条目
  失败时清理已落盘文件，不留半成品。

list_installed
  从 lockfile 与 host 安装目录读取，输出 package_id, remote_url,
  commit_sha, content_hash, installed_at。

uninstall
  输入：package_id
  执行：调用 kernel.package.unload → 删除 install_root 子目录 → 移除 lockfile 条目
  生成审计事件，不可绕过提案。

update
  输入：package_id, target_ref（默认仓库 default branch）
  执行：plan_install 流程的更新版——会比对当前 lockfile 中的 commit_sha，
       生成「从 X 升级到 Y」的提案；apply 时先卸载旧版再装新版。

inspect_lockfile
  描述当前 profile 的 lockfile 状态、未引用包、不一致条目。
```

所有能力是普通能力，能调用 `kernel.outbound.git_fetch` 是因为该包在自己的 manifest 里申报了 `permissions.git_fetch.hosts`，host policy 允许它。

### 提案形态

`apply_install` 不是直接动作。`plan_install` 生成的 proposal_draft 走标准 `kernel.proposal.*` 流程：

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
    - 新增能力包 package_id=...
    - 占用磁盘 ~12.4 MB
    - 申请权限 network.hosts=[…], filesystem_read=[…]
  requires_user_approval: true
  source_ref: official/package-installer-lab/plan_install
```

不审批不安装。审批意味着审批了「这个 commit_sha + 这套权限」，不是审批了「这个仓库以后所有的 commit」。

### 信任模型

```text
钉死  commit SHA      —— 必须，写进事件与 lockfile
钉死  整树 content hash —— 必须，作为 commit SHA 的二次校验
钉死  manifest 内容地址 —— 必须，apply 时与 plan 时不一致就拒绝
签名  git tag GPG / SSH 签名 —— 第一阶段不实现
```

第一阶段不做包签名网络、不做依赖解析、不做依赖图——这些都是延后议题，跟 marketplace 一起延后。

## Lockfile

profile 级，不是 host 级。

每个 profile 维护自己的 lockfile，路径：

```text
<profile-yaml-同目录>/<profile-name>.lock.yaml
```

例如 `profiles/forge-alpha.yaml` 对应 `profiles/forge-alpha.lock.yaml`。

### Lockfile 结构（草案）

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
  - 严禁手工修改 commit_sha 或 content_hash 字段；
  - 这些字段由 installer-lab 写入与维护。
```

profile 级的好处：

- 每个 host 配置（`forge-alpha`、`forge-postgres-example`、`tavern-host` 等）有自己的包集合，不共享。
- profile 切换 = 包集合切换，不会带着上一个 profile 的脏装包。
- profile 是已经有的 host 配置概念，复用它不引入新概念。

不同 host 想共享缓存？走 git 自己的缓存路径（host 可以把多个 profile 的 `install_root` 配成同一个目录），lockfile 互不影响。

## CLI

```bash
ygg package install <github-url> [--ref <branch|tag|sha>] [--profile <name>]
ygg package list-installed [--profile <name>]
ygg package uninstall <package_id> [--profile <name>]
ygg package update <package_id> [--ref <branch|tag|sha>] [--profile <name>]
ygg package inspect-lockfile [--profile <name>]
```

CLI 是 `installer-lab` 的薄客户端。它走公开协议调用能力，跟其他 CLI 命令一样没有特权。

## 安全边界

下面这些条目是任何阶段都不能违反的红线：

- **HTTPS-only。** 任何 SSH、git://、file:// 都直接拒绝。
- **目标 host allowlist 是必填的。** 不允许通配符默认值（`*`）。
- **默认 deny-all。** profile 里不写 `outbound.git.enabled: true` 就装不了任何包。
- **审批路径不可绕过。** `apply_install` 只接收 host-side approved 的 proposal_id，CLI 自动 approve 选项不存在。
- **Plan/Apply 一致性。** apply 时重新 resolve commit_sha 与 content_hash，跟 proposal 里钉的不一致直接 abort。
- **Token 仅 secret_ref。** 私有仓库支持延后；真支持时 token 走 `secret_ref:env:NAME`，host policy 显式 allowlist 解析。
- **磁盘上限。** 超过 `max_clone_size_mb` 中止。
- **重定向 fail-closed。** 跟 `LiveHttpOutboundExecutor` 同款。
- **生成审计。** 每次 fetch、deny、completed、failed 都进事件日志，redaction_state=redacted。
- **没有 post-install 脚本。** 第一阶段不允许包内 hook 在安装时执行任意代码。能力包要在被 `kernel.package.load` 后才有机会跑代码，跟所有别的包同一规则。

## conformance 计划（草案）

第一阶段需要这些用例：

```text
git_fetch.deny_all_default                   默认 profile 调用 git_fetch 全拒
git_fetch.requires_https                     http:// / git@ / file:// 全拒
git_fetch.requires_host_allowlist            未列入 allowlist 的 host 拒绝
git_fetch.fake_executor_returns_fixture      conformance fixture 跑通
git_fetch.audit_no_raw_secrets               审计事件里无 raw token

installer_lab.contract_shape                 契约形状
installer_lab.plan_install_no_apply          plan 阶段不写盘
installer_lab.apply_install_requires_proposal apply 时未审批失败
installer_lab.plan_apply_consistency         commit_sha 不一致 abort
installer_lab.lockfile_round_trip            装一个、读 lockfile、卸载、再读
installer_lab.update_diff_preview            update 生成 X→Y 提案
installer_lab.uninstall_cleans_disk          uninstall 后无残留
installer_lab.no_kernel_namespace_leak       输出不含 kernel.git/repository/dependency
```

真实联网 conformance 走 opt-in 环境变量（仿照 `YGG_TDB_REAL_TESTS=1`）：

```text
YGG_GIT_INSTALL_REAL_TESTS=1
  跑一组指向 github.com/Youzini-afk/<某个 fixture repo> 的真实 fetch
```

默认 CI 不联网。

## 实施顺序

不分 Alpha/Beta/Phase，列动作即可。每条都能独立做完、独立提交、独立验证：

1. 内核：`GitOutboundRequest` / `GitOutboundResponse` 类型，`GitOutboundExecutor` trait，`DenyAllGitOutboundExecutor`，单元测试覆盖默认拒绝。
2. 内核：profile 解析 `outbound.git` 配置，新增审计事件 kind，红线单元测试（HTTPS-only、allowlist 必填、deny-all 默认）。
3. 内核：`FakeGitOutboundExecutor` + 一组 fixture 仓库内容（不联网）；`kernel.outbound.git_fetch` 协议方法；conformance 默认用例覆盖前面所列前 5 条。
4. 普通包：`official/package-installer-lab` skeleton，`describe_install_contract` + `plan_install`（只解析 manifest blob，不 clone full tree），proposal 走通。
5. 普通包：`apply_install` + lockfile 写入，调用 `kernel.package.load`，conformance 用 fake executor 跑通装包闭环。
6. 普通包：`list_installed`、`uninstall`、`inspect_lockfile`、`update`，配套 conformance。
7. 内核：`RealGitOutboundExecutor`（基于 `gix`），HTTPS-only，size cap，timeout，redirect fail-closed；`YGG_GIT_INSTALL_REAL_TESTS=1` opt-in 真实 fetch 用例。
8. CLI 命令、文档收敛、profile 示例 `profiles/forge-with-git-install.example.yaml`。

每一步完成 push 后再做下一步。

## 跟 YdlTavern 的关系

YdlTavern 装扩展是 YdlTavern 自己的事：它在自己的产品逻辑里决定怎么管理 SillyTavern 扩展。如果它需要从 git 装一些 Yggdrasil 能力包来支持自己的功能，可以直接调用 `installer-lab` ——但那条路径上 Yggdrasil 看到的就是普通能力包安装，跟谁调它没关系。

Yggdrasil 这边不会为 YdlTavern 的扩展生态加任何特殊路径。

## 不做

下面这些都不在第一阶段范围内，避免让设计被产品需求拽偏：

- **私有仓库 / token 认证。** 等公开 HTTPS 路径稳定后再做。
- **SSH transport。** 用 HTTPS deploy key 或 token 替代。
- **包签名 / 信任网络。** commit SHA + content hash 已经够用作完整性校验。
- **依赖解析。** 一个 git URL 装一个能力包，不递归装依赖。依赖管理是后续话题。
- **Marketplace / registry。** git URL 即一切。
- **Post-install 脚本。** 永远不在安装阶段执行任意代码。
- **多 host 共享 lockfile。** profile 级，不跨 host。

## 红线

- 不新增 `kernel.git.*` / `kernel.repository.*` / `kernel.package.install_*` / `kernel.dependency.*` 这些 namespace。
- 内核里不出现 git library 的领域类型（commit、tree、refspec 这些都是能力包内部细节）。
- 官方 `installer-lab` 不享受任何特权——同一道权限闸门、同一道审计、同一份审批闸门。任何第三方包都可以写一个等价的安装包来替换它，host policy 允许就能用。
- 装包不依赖具体的 git 实现。`gix` 是当前推荐选择，但 trait 不耦合到任何 crate。

## 接下来

文档落地后开始第 1 步。整个工作不挂在「Alpha + Beta + Phase」名下——它就是一项后台底座工作，做完就完。
