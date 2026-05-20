# Agentic Forge 包编写指南

> [English](./AGENTIC_FORGE_PACKAGE_AUTHORING.en.md) · [中文](./AGENTIC_FORGE_PACKAGE_AUTHORING.md)

本指南描述 Agentic Forge 契约：如何在 Yggdrasil 中构建、运行和替换 agent 包——无需 kernel 特权。

## Agentic Forge 是什么

Agentic Forge 是一个 **package-owned agent runtime 契约**。Agent 是普通包，它们观察 branch、维护 plan graph、调用 scoped capability tool、在 scratch branch 探索、产生 candidate，并通过可审查的 proposal 请求用户 promote。Agent 不进入 kernel ontology。

## Agentic Forge 不是什么

- 不是 chat shell、coding-agent 克隆或 API gateway
- 不是 kernel `agent.*` 命名空间或协议方法
- 不是拥有 ambient authority 的特权 runtime
- 不是 model provider、prompt manager 或 memory store
- 不是 always-on autonomous agent 或 cost optimizer

## Package-Owned Runs

每个 agent run 由一个特定包拥有（`owner_package`）。Run lifecycle 有 9 个状态：

```
created → prepared → running → paused → waiting_for_approval
                  ↘ completed   ↗
                  ↘ failed      ↗
                  ↘ cancelled   ↗
                  ↘ archived
```

包控制 run。Kernel 没有 `kernel.agent.run` 或类似方法。

## Plan Graph

每个 run 维护一个 plan graph：

- **Nodes** 有显式 kind：`observe`、`infer`、`tool_call`、`inspect`、`branch_op`、`compare`、`propose`、`wait`
- **Edges** 连接 nodes
- **status**、**revision**、**input_refs**、**output_refs**
- **approval_policy** 和 **retry_policy**
- **deterministic_mode** 标记

Plan graph 是确定性的。不执行网络。Plan 可以被导出、审查和重放，无副作用。

## Scratch Branch / Candidate / Promote

Agent 在 **scratch branch** 上探索——永远不直接修改 target branch。

1. **`create_candidate`** 产生 branch-aware candidate artifact，含 `scratch_branch_ref`、`target_branch_ref`、`changed_asset_refs`、`confidence`、`uncertainty`、`status`。
2. **`compare_candidate`** 产生 diff summary（scratch vs target），含 **stale 检测**：如果 `target_revision` 不匹配 `current_target_revision`，`stale=true`。
3. **`draft_promote_proposal`** 产生 proposal draft（package-owned `asset.put` 操作），含 `requires_user_approval=true`。永远不直接修改 target。如果 target 是 stale 的，promote 被阻断。
4. **`archive_candidate`** 将 candidate 设置为 `archived`，不修改 target。

## Tool Bridge Scoped Grants

Capability tool bridge（`official/capability-tool-bridge-lab`）提供：

- **`explain_tool_call`**：Scoped grant summary 含 branch-aware context。`no_execution=true`、`no_ambient_authority=true`。
- **`record_tool_observation`**：接受 untrusted tool output（`untrusted=true`）。大输出获取 `asset_ref` 推荐。Raw secret 被阻断。
- **`summarize_tool_risk`**：风险类别：`prompt_injection`、`secret_exfiltration`、`branch_write`、`outbound_expansion`、`nested_delegation`、`large_output`。每项含 typed mitigations。
- **`replay_tool_plan`**：确定性指纹重放。不匹配时标记，绝不静默通过。
- **`plan_toolchain`**：多步 plan-only。每步必须有 `provider_package_id`。嵌套 delegation 无 `explicit_delegation=true` 时阻断。Target branch 写入无 promote grant 时阻断。

## Inference Fallback

Inference-backed agent run 支持 4 种 provider：

| Provider | 行为 |
|----------|------|
| `deterministic` | 默认。根据 objective 产生 `candidate_seed` 或 `proposal_seed`。无网络。 |
| `recorded` | 重放 recorded output。指纹不匹配时标记。 |
| `cloud_adapter_plan` | 返回 `needs_host_policy`。不执行网络。 |
| `local_fake` | Fake local inference。`inference_performed=true`，但无网络。 |

Inference output 根据 allowlist 验证：`candidate_seed`、`proposal_seed`、observation`、`needs_repair`。Forbidden actions（`privilege_escalation`、`auto_promote`、`secret_request`、`target_branch_write`、`unknown_action`）被拒绝。

## Failure Taxonomy

9 种 inference failure kind，含 typed recovery hint：

`rate_limit` · `quota` · `timeout` · `auth` · `network_denied` · `invalid_output` · `malformed_output` · `replay_mismatch` · `policy_reject`

## Third-Party Replacement

Official agentic-forge-lab 是 **ordinary package**——无 kernel 特权，无路由优先。第三方包可以替换它：

1. 创建具有等价 capability shape 的包（如 `thirdparty/agentic-forge`）
2. 创建声明 `replacement_candidates: [official/agentic-forge-lab]` 的 composition
3. 两个包都产生 package-owned shapes：candidates、proposals、plan graphs、working state
4. 两个包都不能直接修改 target branch 或执行网络

参见 `examples/packages/thirdparty-agentic-forge/` 和 `examples/compositions/agentic-forge-replacement/`。

## Secret 安全

- Raw-secret-like 内容被阻断，返回 `redaction_state=unsafe_blocked`
- 任何输出不回显 raw secret
- 使用 `secret_ref` 引用代替嵌入 secret
- Inference output 不能请求 secret（`secret_request` 是 forbidden action）

## Budget 和 Deadline

- `describe_contract` 声明 `run_constraints`，含 budget/deadline 支持
- `start_run` 接受可选 `max_steps` 和 `deadline_ms`
- `cancel_run` 产生一致的 `cancelled` 状态，含 trace events 包含 reason
- 缺少 budget 被诊断，不静默忽略

## Forge Workspace 公共协议

Forge workspace surface（`forge_panel`、`assistant_action`、`home_card`）是公共协议。任何包都可以贡献到这些 surface slot——official 或 third-party。Runtime 不偏好 official 包。

## TypeScript SDK

`sdk/typescript/agentic-forge/` 提供以下 helper：

- Run lifecycle states、plan graph、working state、candidates
- Compare、promote、archive 操作
- Inference node、replay、validation、failure taxonomy
- Tool bridge：risk categories、tool call context、toolchain steps
- Secret 安全：`blockRawSecrets`、`looksLikeRawSecret`、`hasKernelAgentNamespace`

自测：154 项断言，全部确定性，无网络。

## 非目标

- Always-on autonomous agents
- Provider router 或 cost optimizer
- Multi-model tournament
- Shell/fs/git 默认工具
- 自动权限提升
- 直接 target branch mutation
- Chat/coding-agent 作为主要身份
