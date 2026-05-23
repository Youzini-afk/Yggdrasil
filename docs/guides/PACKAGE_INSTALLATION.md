# 包安装

> [English](./PACKAGE_INSTALLATION.en.md) · [中文](./PACKAGE_INSTALLATION.md)

Yggdrasil 的安装系统让用户从 GitHub 或本地路径安装能力包和项目，并保持可重现、可审计、可撤销。
本文档描述安装流程、原生/外部项目检测、清单字段、锁文件、文件系统约定和命令行用法。

## 目标

- 让普通用户可以用一条命令安装能力包。
- 让创作者可以声明 package 依赖，而不是复制 profile 片段。
- 让安装结果可以被 lockfile 重现。
- 让每一次新增权限都经过用户同意。
- 让失败路径默认安全，不产生半写入 profile。
- 让卸载只移除 profile 引用，不破坏内容寻址 store。

## 设计原则

- 内核不知道 git。
- git 通过 `official/git-tools-lab`（能力包）走 `kernel.v1.outbound.execute` 边界。
- 安装编排在 `official/install-lab`（能力包）中，不在 kernel。
- 默认拒绝：HTTPS-only，拒绝 `ssh://`、`git://`、`file://`。
- 默认拒绝：URL 不能含 username/password。
- 完整性：每个包记录 commit、tree hash、manifest hash。
- 签名（可选）：GPG 签名标签验证，公钥白名单。
- 一致性：lockfile + 不可变内容寻址存储。
- 审计：用户授权的能力、网络、secret 都记录在 lockfile 中。
- 同意：新增或扩展授权时弹窗。
- 官方包无特权：安装包、git 工具、完整性工具都按普通清单加载。

## 用户流程

### 安装

```bash
# 简单情况
yg install github.com/user/yggdrasil-package

# 原生项目（仓库根目录有 project.yaml）
yg install github.com/Youzini-afk/Yggdrasil-Tavern

# 锁定版本（推荐）
yg install github.com/user/yggdrasil-package#v1.2.0

# 本地路径（开发）
yg install ./packages/my-package

# 要求签名标签（发布/受控环境）
yg install <url> --require-signed

# 非交互式（CI）
yg install <url> --yes

# 严格 conformance gating
yg install <url> --strict

# 外部项目策略
yg install github.com/user/external-app --wrap-as-adapter
yg install github.com/user/external-app --workspace-only
```

### 其他命令

```bash
yg list-installed [--profile <name>]
yg project list
yg project info <id>
yg project status <id>
yg project start <id>
yg project stop <id>
yg uninstall <package-id-or-project-id> [--profile <name>]
yg update [<package-id>]      # 检查上游，安装更新
yg lockfile [--check]         # 验证 lockfile 与 store 一致
```

### Profile 与 data dir

默认 profile 名为 `default`。
命令可通过 `--profile <name>` 操作不同 profile。
命令可通过 `--data-dir <path>` 覆盖数据目录，适合测试和 CI。

```bash
yg install ./packages/dev --profile alpha --data-dir /tmp/ygg-alpha --yes
yg list-installed --profile alpha --data-dir /tmp/ygg-alpha
```

安装相关 flag：

- `--require-signed`：要求 Git tag 签名可验证；默认不强制签名。
- `--strict`：conformance 失败时阻断安装；默认只警告并继续。
- `--yes`：非交互式批准同意提示。
- `--profile <name>`：选择要更新的 profile。
- `--data-dir <path>`：覆盖 `~/.yggdrasil` 数据目录，适合测试和 CI。
- `--wrap-as-adapter`：外部项目安装时生成/使用 adapter package。
- `--workspace-only`：外部项目只作为 agent workspace 注册，不包装。


## 原生 vs 外部项目检测

`yg install <url>` 会先检查源根目录是否存在 `project.yaml`。

| 检测结果 | 行为 |
|---|---|
| 有有效 `project.yaml` 且 `project.type: yggdrasil_native` | 安装为原生 Yggdrasil 项目，注册到 `ProjectRegistry`，写入 `~/.yggdrasil/projects/<id>/`，Home 显示项目卡片。 |
| 有 `project.yaml` 但无效 | fail-closed，要求修正 descriptor。 |
| 没有 `project.yaml` | 进入外部项目 wizard。 |

原生项目的 `project.yaml` 引用项目需要的 package manifest，并给出 `entry_surface_id`。该 surface 应由其中一个 package 贡献，通常是 `slot: experience_entry`。

详见 [`PROJECT_MODEL.md`](PROJECT_MODEL.md)。

## 外部项目 wizard

外部项目不是为 Yggdrasil 写的仓库。安装器会展示检测结果（语言、包管理器、入口、生命周期风险），然后让用户选择：

1. **Wrap with adapter**：生成 adapter package，把外部项目作为受控能力或 surface 接入。适合长期使用。
2. **Workspace only**：只注册为 agent workspace，不生成包装层。适合临时分析、修改、迁移。
3. **Cancel**：不安装。

无 TTY 且没有显式 flag 时，默认选择 `workspace-only`，避免自动生成 adapter 代码。

### `--wrap-as-adapter`

强制选择包装路径。安装器会通过外部项目 intake / adapter 规划生成 adapter manifest preview，并在用户同意后写入项目记录。adapter 仍是普通能力包，没有内核特权。

### `--workspace-only`

强制选择工作区路径。Yggdrasil 只记录项目来源、工作区路径、检测 metadata 和后续 agent 操作策略，不声明该外部项目已成为 Yggdrasil 能力包。

## 清单 `requires` 字段

包通过 `manifest.yaml` 声明依赖：

```yaml
requires:
  - id: "official/composition-lab"
    source:
      kind: internal
    version: ">=1.0.0"

  - id: "third-party/cool-tool"
    source:
      kind: git
      url: "https://github.com/user/cool-tool"
      ref: "v1.2.0"
    version: ">=1.0.0"
    minimum_signed_by: ["FA9C5BC2..."]

  - id: "local/dev-helper"
    source:
      kind: local
      path: "../dev-helper"
```

字段：

- `id`：包 id，必须与解析出的 `manifest.id` 匹配。
- `source`：来源，支持 `internal`、`git`、`local`。
- `version`：语义版本约束，例如 `""`、`">=1.0.0"`、`"^2.1"`、`"=1.2.3"`。
- `minimum_signed_by`：可选 GPG fingerprint 白名单，强制签名。

`requires` 是安装数据，不授予运行时权限。
运行时权限仍由 `permissions`、bindings 和 capability handles 决定。
`consumes` 声明能力需求；`requires` 声明包依赖。

## 锁文件

锁文件位置：

```text
~/.yggdrasil/profiles/<name>.lock.toml
```

详见 [`../spec/v1/LOCKFILE_FORMAT.md`](../spec/v1/LOCKFILE_FORMAT.md)。

Lockfile 记录：

- profile 名称与 manifest hash；
- 每个安装包的 id、version、source、ref、commit；
- `manifest_hash` 与 `tree_hash`；
- store 路径；
- 签名状态与签名 fingerprint；
- 已授权能力、网络、secret；
- 解析后的直接依赖边。

这让工具可以回答：

- 当前包从哪里来；
- 为什么这个包被安装；
- 当前安装是否仍与 lockfile 一致；
- 更新会影响哪些下游包；
- 用户已经同意过哪些权限。

## 文件系统布局

```text
~/.yggdrasil/
├── store/              # 不可变内容寻址存储
│   ├── sha256-abc.../
│   └── sha256-def.../
├── profiles/           # 可变 profile + lockfile
│   ├── default.yaml
│   ├── default.lock.toml
│   └── alpha.yaml
├── keys/               # 受信任的 GPG 公钥
│   └── trusted-keys.asc
└── cache/git/          # git fetch 缓存
```

数据目录选择顺序：

1. `YGG_DATA_DIR`；
2. `XDG_DATA_HOME` 下的 Yggdrasil 目录；
3. `~/.yggdrasil`。

CLI 的 `--data-dir` 优先级最高，主要用于测试、CI 和一次性演示。

## 安装流程详解

```text
yg install github.com/user/repo#v1.0
            ↓
1. URL 解析（parse_install_url）
            ↓
2. 加载现有 lockfile（如果存在）
            ↓
3. install-lab.resolve_plan
   ├─ git-tools-lab.resolve_ref → commit_sha
   ├─ git-tools-lab.fetch_tree → 临时目录
   ├─ git-tools-lab.read_signed_tag → pgp_signature
   ├─ integrity-lab.compute_manifest_hash
   ├─ integrity-lab.compute_tree_hash
   ├─ integrity-lab.verify_gpg_signature（如有签名）
   ├─ ygg-core::conformance::run_checks（静态）
   └─ 递归 manifest.requires（循环检测）
            ↓
4. 显示计划（人类可读 + 签名状态 + 完整性哈希）
            ↓
5. 同意提示（新/扩展授权）
   ├─ TTY：交互式 dialoguer 提示
   ├─ --yes：自动批准
   └─ 无 TTY 且无 --yes：错误
            ↓
6. install-lab.execute_plan
   ├─ 验证同意覆盖计划授权
   ├─ 重新 fetch 到 staging
   ├─ 原子 rename 到 store
   ├─ 更新 profile YAML（原子）
   └─ 写入 lockfile（原子）
            ↓
7. 完成
```

## 安全模型

### HTTPS-only

Git URL 默认只接受 HTTPS。
`ssh://`、`git://`、`file://` 全部拒绝。
URL 含 username/password 也拒绝，避免 credential 进入日志、审计或 lockfile。

### 路径校验

`fetch_tree` 的 `dest_dir` 必须是绝对路径，不能含 `..` 组件。
写入树时拒绝危险条目，例如 `.git`、路径分隔符和父目录引用。

### 原子写入

所有 profile、lockfile、store 写入都采用 tmp + rename 模式。
中途崩溃可能留下临时目录，但不会让 store、profile、lockfile 处于半写入状态。

### 不可变 store

`~/.yggdrasil/store/` 是内容寻址存储。
内容写入后不再修改。
卸载只移除 profile 与 lockfile 引用，store 目录保留为孤立内容。
未来 `yg gc` 命令会回收孤立 store 目录。

### 默认安全基线

默认行为对齐 cargo/npm/pip 这类包管理器的技术基线：HTTPS-only、原子写入、内容哈希始终启用；签名验证和 conformance gating 是显式 opt-in。

- HTTPS-only 与 URL credential 拒绝始终启用。
- content hash（tree hash / manifest hash）始终记录。
- profile、lockfile、store 写入始终使用原子写入。
- 签名验证通过 `--require-signed` 启用。
- conformance 阻断通过 `--strict` 启用。

### 签名验证

默认不要求 Git 包带 GPG 签名标签，但如果存在签名仍会记录状态。
`minimum_signed_by` 字段强制特定 fingerprint。
`--require-signed` 要求签名可验证，适合发布、受控环境或组织策略。
底层完整性工具基于 `sequoia-openpgp`，支持常见 RSA / Ed25519 签名材料。

### Conformance gating

安装前会运行静态 v1 conformance 检查。
默认行为是 warning-only：失败会显示在安装计划中，但不阻断安装。
`--strict` 会把 conformance 失败提升为安装阻断，适合 CI、发布或组织策略。

### API key 与 secret

安装只记录用户同意的 `secret_ref` 权限，不采集 raw API key。
API key 管理见 [`SECRET_MANAGEMENT.md`](SECRET_MANAGEMENT.md)：桌面端推荐 `secret_ref:store:*`，开发和 CI 可继续使用 `secret_ref:env:*`。

### 同意审计

Lockfile 的 `granted_capabilities`、`granted_network`、`granted_secrets` 字段记录用户实际同意的授权。
后续安装或更新会与既有授权对比，只对新增或扩展授权弹窗。

## 卸载

```bash
yg uninstall fixture/pkg-local
```

卸载会：

1. 从 profile YAML 移除该包；
2. 从 lockfile 移除对应 entry；
3. 保留 store 内容；
4. 原子写回 profile 与 lockfile。

卸载不会删除其他包仍引用的依赖。
如果未来加入依赖反查，CLI 可以提示哪些包仍需要当前包。

## 更新

```bash
yg update
yg update third-party/cool-tool
```

更新会检查上游 ref，解析新计划，并重新执行完整性、签名、conformance 和同意检查。
如果权限没有变化，用户不需要重复确认旧授权。
如果新增网络、secret 或 capability 权限，必须重新同意。

## 漂移检测

```bash
yg lockfile --check
```

该命令会：

1. 读取 lockfile；
2. 验证每个 `LockEntry` 的 store 路径存在；
3. 重新计算 `manifest_hash` 并与 lockfile 比对；
4. 重新计算 `tree_hash` 并与 lockfile 比对；
5. 报告任何漂移。

非零退出码用于 CI：漂移即失败。

## 实现细节

参考：

- `crates/ygg-core/src/manifest.rs`（`PackageDependency`、`DependencySource`）
- `crates/ygg-core/src/lockfile.rs`（`Lockfile`、`LockEntry`）
- `crates/ygg-core/src/paths.rs`（filesystem layout）
- `crates/ygg-core/src/conformance.rs`（静态检查可重用）
- `crates/ygg-runtime/src/inproc/install_lab.rs`（orchestrator）
- `crates/ygg-runtime/src/inproc/git_tools_lab.rs`（gix-based git）
- `crates/ygg-runtime/src/inproc/integrity_lab.rs`（sequoia GPG + sha256）
- `crates/ygg-cli/src/commands/install.rs`（CLI 入口）
- `crates/ygg-cli/src/install/consent.rs`（同意提示）
- `crates/ygg-cli/src/install/url_parser.rs`（URL 解析）

## Conformance 覆盖

Round 10A 覆盖：

- git URL 与路径拒绝；
- signed tag fixture；
- tree hash、manifest hash、GPG verify、fingerprint；
- resolve plan、execute plan、uninstall、list、lockfile drift；
- transitive dependency 与循环依赖；
- conformance gating、strict block、lenient warning、transitive propagation；
- `install.real_github_smoke` 真实 GitHub opt-in smoke。

默认 conformance 不联网。
真实 GitHub smoke 需要显式设置：

```bash
YGG_GIT_INSTALL_REAL_TESTS=1 cargo run -p ygg-cli -- conformance --case install.real_github_smoke
```

## 限制（Round 10A）

- Sigstore keyless 验签：推迟（无 git 标签约定）。
- Tauri UI 安装：推迟（仅 CLI）。
- 中央 marketplace：不做（违反平台哲学）。
- 自动更新守护进程：推迟（`yg update` 手动）。
- 二进制包分发：推迟（仅源/git）。
- 跨 profile 包共享语义：推迟。
- `yg gc` 孤立 store 回收：Round 11+。

## 推荐实践

- 发布包时使用不可变 tag，不要让用户安装浮动分支。
- 对 GitHub 包启用签名标签。
- 在 `requires` 中固定上游 ref，并使用合理 version constraint。
- 在 CI 中运行 `yg lockfile --check`。
- 本地开发可直接使用 `yg install <url>`；发布或受控环境按需加 `--require-signed` 与 `--strict`。
- 为 Yggdrasil 原生体验优先提供根目录 `project.yaml`，而不是只发布零散 package manifest。
- 对新增网络和 secret 权限写清楚 purpose，方便用户同意。
