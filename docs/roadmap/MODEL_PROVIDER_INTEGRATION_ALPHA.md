# Model Provider Integration Alpha

> [English](./MODEL_PROVIDER_INTEGRATION_ALPHA.en.md) · [中文](./MODEL_PROVIDER_INTEGRATION_ALPHA.md)

这是执行期临时计划。完成后删除本文件，并把 durable 结果收敛进 README、`docs/ALPHA_STATUS.md`、`docs/roadmap/NEXT_STEPS.md`、`docs/spec/CONFORMANCE_MATRIX.md` 和 guide。

目标：以普通能力包实现多 provider 模型接入，覆盖 OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks 的真实 API 差异；默认 conformance 用 fake/local executor，不依赖外网或真实 key；手动真实调用 opt-in。不是中转站、不是计费系统，不新增 kernel model/prompt/chat 语义。

## M0 — Research Ledger ✅

- 新增 `integrations/model-providers/` ledger。
- 固定 provider matrix、stream compatibility、error taxonomy。
- 记录 `new-api` 与 TavernHeadless 可吸收/不可吸收经验。

## M1 — Model Provider Adapter SDK

- 新增 `sdk/typescript/model-provider-adapter`。
- 定义 provider profile、canonical request/response、normalized stream events、usage/cost/error metadata。
- 提供 OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks normalization helpers。

## M2 — `official/model-provider-lab` no-network normalization

- 新增普通官方包，先提供 `list_supported_families`、`validate_profile`、`normalize_request`、`explain_error`、`echo`。
- 不出网、不做真实 inference。

## M3 — Host Outbound Executor Boundary

- 新增 content-free outbound executor abstraction：request/response/fake executor/local mock 支持。
- 保持默认 deny/fake；真实出网需要显式 opt-in。
- 强制 network allowlist、secret_ref、redacted audit、timeout/cancel。

## M4 — OpenAI / Anthropic / Gemini invoke adapters

- 在 `model-provider-lab` 中实现三类非兼容代表的 fake/local invoke path。
- 支持手动真实调用路径，但不进入默认 conformance。

## M5 — OpenAI-compatible / OpenRouter / DeepSeek / xAI / Fireworks presets

- 增加 provider presets、base URL/header quirks、usage/error mapping。
- OpenAI-compatible 是 adapter family，不是唯一协议。

## M6 — Streaming normalization

- 将 delta SSE、semantic SSE、typed chunk stream 归一为 provider package normalized stream events，再包装为 `StreamFrameEnvelope`。
- 覆盖 terminal/error/usage/cancel/timeout。

## M7 — Examples, conformance, durable docs, cleanup

- 新增 provider profile examples、manual live smoke docs、conformance。
- 新增 `docs/guides/MODEL_PROVIDER_INTEGRATION.md` / `.en.md`。
- 删除本临时计划。

## 非目标

- 不做用户余额、计费、渠道后台、admin UI。
- 不托管平台代理 key。
- 不新增 `kernel.model.*`、`kernel.prompt.*`、`kernel.chat.*`、`kernel.embedding.*`。
- 不让官方 provider 包获得隐式 secret/network/routing/UI 特权。
