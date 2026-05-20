# 分享与分发指南

> [English](./SHARING_DISTRIBUTION.en.md) · [中文](./SHARING_DISTRIBUTION.md)

本文档说明 Yggdrasil 中可分享、可复现、可导入的 composition 和 session 分发机制。这是 Experience Beta 6 的交付成果，由 `official/sharing-lab` 普通能力包提供。

## 核心原则

- **先分享，后市场**：当前只做本地/文件级的分享 proof——export/import composition bundle、branch/session bundle manifest、package-set lockfile、compatibility/migration report、AI disclosure metadata bundle、read-only shared session manifest 和 async fork sharing plan。
- **不做 marketplace**：不引入包签名网络、依赖解析器经济、托管计费。分发是本地文件交换，不是商业市场。
- **不做 `kernel.sharing.*`**：分享是 package-owned 行为，不是内核能力。
- **不做 raw secrets**：所有 bundle 中只允许 `secret_ref` 引用，不存储原始密钥。
- **不要求公网**：所有分享 proof 是本地文件，不依赖远端服务或公网访问。

## 分享契约

`official/sharing-lab` 提供 9 项能力和 3 个 surface（forge_panel、assistant_action、home_card）。核心契约如下：

| 能力 | 用途 |
|---|---|
| `describe_sharing_contract` | 描述分享契约：能力列表、surface、输出形状、红线约束 |
| `export_composition_bundle` | 导出 composition 为自包含 bundle：manifest + lockfile + disclosure |
| `import_composition_bundle` | 导入 bundle，验证形状、兼容性、无 raw secret 约束 |
| `create_branch_session_bundle` | 创建 branch/session bundle manifest，用于分享特定 session 状态 |
| `create_package_set_lockfile` | 创建 package-set lockfile，锁定包版本和内容地址 |
| `compatibility_report` | 生成兼容性/迁移报告，对比两个 bundle 版本或包集 |
| `ai_disclosure_bundle` | 生成 AI 披露元数据 bundle，标记内容来源 |
| `read_only_share_manifest` | 创建只读共享 session manifest（本地/文件级 proof） |
| `async_fork_share_plan` | 创建异步 fork 分享计划（本地 proof，延迟/异步 session fork） |

## Bundle 形状

### Composition Bundle

```json
{
  "bundle_id": "bundle:<composition_id>:<content_address>",
  "format_version": "1",
  "composition_id": "...",
  "composition_manifest": { ... },
  "package_set_lockfile": {
    "lockfile_id": "lockfile:<content_address>",
    "format_version": "1",
    "packages": [
      { "package_id": "...", "version": "...", "content_address": "fnv1a64:..." }
    ],
    "content_address": "fnv1a64:..."
  },
  "ai_disclosure": {
    "disclosure_id": "disclosure:<bundle_id>",
    "items": [
      { "content_ref": "...", "disclosure_kind": "ai_generated|ai_assisted|human_created|mixed", "description": "..." }
    ],
    "content_address": "fnv1a64:..."
  },
  "no_marketplace_fields": true,
  "no_billing_fields": true,
  "no_signing_network_fields": true
}
```

### Branch/Session Bundle

```json
{
  "bundle_id": "branch-bundle:<session_id>:<branch_ref>:<content_address>",
  "format_version": "1",
  "session_id": "...",
  "branch_ref": "branch:main",
  "sequence": 42,
  "content_address": "fnv1a64:...",
  "ai_disclosure": { ... }
}
```

### Package-Set Lockfile

```json
{
  "lockfile_id": "lockfile:<content_address>",
  "format_version": "1",
  "packages": [
    { "package_id": "...", "version": "...", "content_address": "fnv1a64:..." }
  ],
  "content_address": "fnv1a64:..."
}
```

### Compatibility Report

```json
{
  "report_id": "compat-report:<source>:<content_address>",
  "source_ref": "bundle:v1",
  "target_ref": "bundle:v2",
  "status": "compatible|minor_incompatibility|major_incompatibility|migration_required",
  "incompatibilities": [
    { "package_id": "...", "kind": "missing_in_target|version_mismatch|added_in_target", "severity": "minor|major" }
  ],
  "migration_steps": [ { "action": "...", "package_id": "..." } ]
}
```

## AI 披露

每个 bundle 都可以附带 AI disclosure metadata，标记内容来源：

| `disclosure_kind` | 含义 |
|---|---|
| `ai_generated` | 内容完全由 AI 生成 |
| `ai_assisted` | 人类创作 + AI 辅助 |
| `human_created` | 人类原创内容 |
| `ai_reviewed` | 人类创作 + AI 审核 |
| `mixed` | 混合来源 |
| `undisclosed` | 未披露来源 |

## 只读共享与异步 Fork

**只读共享**（`read_only_share_manifest`）：创建一个 session 的只读快照证明，可以被他人查看但不能修改。`share_scope: local_file`，`no_remote_service: true`。

**异步 Fork 分享**（`async_fork_share_plan`）：创建一个异步 fork 计划，允许接收者后续 fork 出自己的 session。状态为 `draft`，`plan_only: true`，需要用户审批。

## 红线

以下行为在分享契约中明确禁止：

- ❌ Marketplace 字段（`marketplace_id`、`marketplace_category`）
- ❌ 计费字段（`billing_token`、`payment_method`、`subscription`）
- ❌ 签名网络字段（`signing_network`、`license_key`）
- ❌ Raw secrets（`api_key`、`token`、`password` 原文值；只允许 `secret_ref` 引用）
- ❌ 内核分享命名空间（`kernel.sharing.*`、`kernel.marketplace.*`、`kernel.billing.*`）
- ❌ 公网或远端服务依赖

## 示例

完整示例见 `examples/bundles/playable-creation-board-composition-bundle/`，包含：
- `bundle.json` — composition bundle + lockfile + compatibility report + AI disclosure
- `branch-session-bundle.json` — branch/session bundle manifest
- `read-only-share-manifest.json` — 只读共享 session manifest
- `async-fork-share-plan.json` — 异步 fork 分享计划

## 验证

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

Conformance 包含 10 个 sharing-lab 用例（260 总计），覆盖契约形状、export/import、lockfile、compatibility report、AI disclosure、只读共享、异步 fork、红线约束。
