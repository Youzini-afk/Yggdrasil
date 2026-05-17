# Model Inference Prerequisites

> [English](./MODEL_INFERENCE_PREREQUISITES.md) · [中文](./MODEL_INFERENCE_PREREQUISITES.zh-CN.md)

Model Connectivity Kit Alpha 刻意停在真实 model execution 之前。未来的 `official/model-inference-lab` 或等价能力包族，必须等以下前置条件被明确并纳入 conformance 后再开始。

## 必需的平台契约

1. **Secret resolution**
   - Profile assets 可以引用 secrets，但 raw secrets 不得出现在 events、projections、logs、UI 或 assistant proposals 中。
   - Host 需要公开的 secret-reference capability 或 policy surface。
   - **Phase S1 progress**：`SecretRef` 类型支持 `secret_ref:`、`secretRef:`、`secret-ref:`、`host:` patterns。已有 `HostSecretResolver` trait 和 `DenyAllSecretResolver` placeholder。Proposals 与 asset metadata 中的 raw-secret blocking 已实现。官方包没有 bypass。Permission grants 可 rehydrate。生产级 vault integration 仍属于 host-level 后续工作。

2. **Network permission**
    - Packages 需要按 destination、method 与 purpose 显式获得 network permissions。
    - 任何包都不能因为是官方包就推断拥有 network permission。
    - **Phase S2 progress**：Manifest `permissions.network` 支持结构化 `declarations`（host、methods、purpose）和扁平 `hosts` 向后兼容。Runtime `check_network_policy` 和 `check_and_audit_outbound` 强制执行 allowlist。官方包无绕过。被拒绝的请求写入 `kernel/outbound.denied`；被允许的请求写入 `kernel/outbound.request` 并带 redacted audit。

3. **Request/response audit**
    - 每个 outbound request 都需要 principal、package id、capability id、provider family、route id、redaction state 与 cost/usage placeholders。
    - Raw prompts/responses 在 audit persistence 前需要 redaction policy。
    - **Phase S2 progress**：`OutboundAuditRecord` 捕获 principal、package_id、capability_id、destination_host、method、purpose、redaction_state、secret_refs_used、usage/cost 占位符、status/error。`RedactionState` 枚举：`not_captured`、`redacted`、`policy_ref`、`unsafe_blocked`、`explicitly_approved`。默认为 `redacted`——raw body/header/prompt/response 不会被保存。可通过 `kernel.outbound.audit` 检查。

4. **Streaming and cancellation**
   - Streaming chunks 需要公开 protocol shape。
   - Cancellation/timeout 行为必须 deterministic 且有测试覆盖。
   - **Phase S3 progress**：`StreamFrameEnvelope` 定义通用内容无关的 frame 类型（start/chunk/progress/end/error/cancelled/timeout），包含 invocation_id、stream_id、sequence、redaction_state 和 timestamp/metadata。`StreamRegistry` 追踪进行中的 invocation，支持 start/append/end/cancel/timeout 生命周期。`kernel.capability.stream` 和 `kernel.capability.cancel` 已 partial dispatched。按序发出 kernel 事件。Cancel/timeout 阻断后续 chunk。非 streaming 能力（streaming=false）被拒绝。未添加 model/agent 方法。

5. **Usage accounting**
   - Provider usage units 必须 normalize，同时保留 provider-specific details。
   - Cost estimates 必须标记为 estimated，除非 provider confirmed。

6. **Provider error taxonomy**
   - Authentication、rate limit、quota、timeout、model not found、malformed request 与 provider outage errors 必须映射为稳定的 package-level diagnostics。

7. **Data redaction and approval**
   - Assistant-mediated inference 在把 user/project data 发送出 host boundary 时必须 approval-gated。
   - Redaction policies 必须能通过 public surfaces 检查。
    - **Phase S1/S2 progress**：已增加通用 redaction scanner/helper，用于 trusted proposal/asset metadata paths；Outbound audit records 使用 `RedactionState` 枚举（`not_captured`/`redacted`/`policy_ref`/`unsafe_blocked`/`explicitly_approved`）。Content/description/title/reason fields 暂不扫描以避免误伤。

## Deferred capabilities

未来 inference packages 可以添加：

- generate request planning；
- non-streaming generation；
- streaming generation；
- embedding calls；
- provider model discovery；
- tool-call mediation；
- usage reports；
- safety/redaction previews。

这些都不属于 Model Connectivity Kit Alpha。
