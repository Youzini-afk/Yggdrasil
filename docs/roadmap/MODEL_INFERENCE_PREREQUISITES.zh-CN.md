# Model Inference Prerequisites

> [English](./MODEL_INFERENCE_PREREQUISITES.md) · [中文](./MODEL_INFERENCE_PREREQUISITES.zh-CN.md)

Model Connectivity Kit Alpha 刻意停在真实 model execution 之前。未来的 `official/model-inference-lab` 或等价能力包族，必须等以下前置条件被明确并纳入 conformance 后再开始。

## 必需的平台契约

1. **Secret resolution**
   - Profile assets 可以引用 secrets，但 raw secrets 不得出现在 events、projections、logs、UI 或 assistant proposals 中。
   - Host 需要公开的 secret-reference capability 或 policy surface。

2. **Network permission**
   - Packages 需要按 destination、method 与 purpose 显式获得 network permissions。
   - 任何包都不能因为是官方包就推断拥有 network permission。

3. **Request/response audit**
   - 每个 outbound request 都需要 principal、package id、capability id、provider family、route id、redaction state 与 cost/usage placeholders。
   - Raw prompts/responses 在 audit persistence 前需要 redaction policy。

4. **Streaming and cancellation**
   - Streaming chunks 需要公开 protocol shape。
   - Cancellation/timeout 行为必须 deterministic 且有测试覆盖。

5. **Usage accounting**
   - Provider usage units 必须 normalize，同时保留 provider-specific details。
   - Cost estimates 必须标记为 estimated，除非 provider confirmed。

6. **Provider error taxonomy**
   - Authentication、rate limit、quota、timeout、model not found、malformed request 与 provider outage errors 必须映射为稳定的 package-level diagnostics。

7. **Data redaction and approval**
   - Assistant-mediated inference 在把 user/project data 发送出 host boundary 时必须 approval-gated。
   - Redaction policies 必须能通过 public surfaces 检查。

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
