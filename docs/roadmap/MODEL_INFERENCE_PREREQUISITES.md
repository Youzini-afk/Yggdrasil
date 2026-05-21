# Model Inference Prerequisites

> [English](./MODEL_INFERENCE_PREREQUISITES.en.md) · [中文](./MODEL_INFERENCE_PREREQUISITES.md)

Model Connectivity Kit 刻意停在真实 model execution 之前。后续工作会把这些前置条件落实为普通能力包、SDK 和 host outbound boundary。它不会变成中转站、计费系统或内核里的模型本体。

## 必需的平台契约

1. **Secret resolution**
   - Profile 资产可以引用 secret，但 raw secret 不得出现在事件、projection、日志、UI 或 assistant 提案中。
   - Host 需要公开 secret-reference 能力或 policy surface。
   - `SecretRef` 类型支持 `secret_ref:`、`secretRef:`、`secret-ref:`、`host:` patterns。已有 `HostSecretResolver` trait 和 `DenyAllSecretResolver` placeholder。
   - 提案与资产 metadata 已阻断 raw secret。官方包没有 bypass。权限 grants 可 rehydrate。生产级 vault integration 仍属于 host-level 后续工作。
   - TypeScript `secretRef()`、`isValidSecretRef()`、`looksLikeRawSecret()`、`isSecretFieldName()` helper 位于 `sdk/typescript/secure-execution`。`--template networked` 演示 `secretRef` 用法。`examples/packages/faux-model-readiness/` 使用 `secret_ref` 引用凭证。

2. **Network permission**
    - 包需要按 destination、method 与 purpose 显式获得网络权限。
    - 任何包都不能因为是官方包就推断拥有网络权限。
    - 清单 `permissions.network` 支持结构化 `declarations`（host、methods、purpose）和扁平 `hosts` 向后兼容。Runtime `check_network_policy` 和 `check_and_audit_outbound` 强制执行 allowlist。
    - 官方包无绕过。被拒绝的请求写入 `kernel/outbound.denied`；被允许的请求写入 `kernel/outbound.request` 并带脱敏审计。
    - TypeScript `NetworkDeclaration` 类和 `OutboundAuditHelper` 位于 `sdk/typescript/secure-execution`。`--template networked` 生成带网络声明和审计 helper 用法的包骨架。`examples/packages/faux-model-readiness/` 声明网络权限并返回 discovery plans。

3. **Request/response audit**
    - 每个 outbound request 都需要身份、package id、capability id、provider family、route id、redaction state 与 cost/usage placeholder。
    - Raw prompt/response 在审计持久化前需要脱敏 policy。
    - `OutboundAuditRecord` 捕获 principal、package_id、capability_id、destination_host、method、purpose、redaction_state、secret_refs_used、usage/cost 占位符、status/error。
    - `RedactionState` 枚举包括 `not_captured`、`redacted`、`policy_ref`、`unsafe_blocked`、`explicitly_approved`。默认是 `redacted`，raw body/header/prompt/response 不会被保存。可通过 `kernel.outbound.audit` 检查。

4. **Streaming and cancellation**
   - 流式 chunk 需要公开协议形状。
   - 取消和超时行为必须稳定，并有测试覆盖。
   - `StreamFrameEnvelope` 定义通用内容无关的 frame 类型（start/chunk/progress/end/error/cancelled/timeout），包含 invocation_id、stream_id、sequence、redaction_state 和 timestamp/metadata。
   - `StreamRegistry` 追踪进行中的 invocation，支持 start/append/end/cancel/timeout 生命周期。`kernel.capability.stream` 和 `kernel.capability.cancel` 已部分分发。内核事件按序发出。Cancel/timeout 会阻断后续 chunk。非流式能力（streaming=false）会被拒绝。未添加 model/agent 方法。
   - TypeScript `StreamFrameClient` helper 位于 `sdk/typescript/secure-execution`，提供 client-side faux frame 构造，支持完整生命周期。`--template streaming` 生成演示 `StreamFrameClient` 用法的包骨架。`examples/packages/faux-model-readiness/` 和 `examples/packages/faux-agent-readiness/` 用 faux frames 证明流式底座形状，不做真实 model inference。

5. **Usage accounting**
   - Provider usage units 必须规范化，同时保留 provider-specific details。
   - Cost estimate 必须标记为 estimated，除非 provider confirmed。

6. **Provider error taxonomy**
   - Authentication、rate limit、quota、timeout、model not found、malformed request 与 provider outage errors 必须映射为稳定的 package-level diagnostics。

7. **Data redaction and approval**
   - Assistant-mediated inference 把用户或项目数据发送出 host boundary 时，必须经过 approval gate。
   - 脱敏 policy 必须能通过 public surfaces 检查。
   - 已增加通用 redaction scanner/helper，用于 trusted proposal/asset metadata paths。Outbound audit records 使用 `RedactionState` 枚举（`not_captured`/`redacted`/`policy_ref`/`unsafe_blocked`/`explicitly_approved`）。Content/description/title/reason fields 暂不扫描，以避免误伤。

## Deferred capabilities

未来 inference package 可以添加：

- generate request planning；
- non-streaming generation；
- streaming generation；
- embedding calls；
- provider model discovery；
- tool-call mediation；
- usage reports；
- safety/redaction previews。

这些都不是当前前置条件的一部分。
