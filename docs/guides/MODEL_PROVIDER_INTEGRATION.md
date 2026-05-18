# Model Provider Integration

> [English](./MODEL_PROVIDER_INTEGRATION.en.md) · [中文](./MODEL_PROVIDER_INTEGRATION.md)

本指南记录 Yggdrasil 的 model provider 接入方式。它不是中转站、不是计费系统、不是 provider 后台，也不是内核模型抽象。模型接入必须作为普通能力包工作，并遵守同一套 manifest、权限、`secret_ref`、outbound audit、stream/cancel 和 conformance 边界。

## 当前交付

Model Provider Integration Alpha 已完成：

- `integrations/model-providers/` 保存 provider research ledger、provider matrix、stream compatibility notes 和 error taxonomy。
- `sdk/typescript/model-provider-adapter` 提供纯 TypeScript adapter，用于 provider profile、请求归一化、错误分类和 stream event 解析；它不出网、不做计费、不访问私有 runtime。
- `official/model-provider-lab` 是普通官方能力包，覆盖 OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks 八类 provider family。
- `official/model-provider-lab` 能力包括：`list_supported_families`、`validate_profile`、`normalize_request`、`invoke`、`normalize_stream`、`explain_error`、`echo`。
- `invoke` 当前是 fake/local provider adapter path：产出 provider 形状的 response 和可审计 `outbound_request_shape`，但 `network_performed=false`、`inference_performed=false`、`executor_kind=fake_local`。
- Host 侧已有 content-free `OutboundExecutor` boundary，默认 deny-all，并有 fake executor conformance；这证明 request shape 可以走 host policy/audit 边界，但不声称拦截 subprocess 任意联网。
- `normalize_stream` 将 delta SSE、semantic SSE、typed chunk stream 归一为 `StreamFrameEnvelope` 风格 frames：`start`、`chunk`、`progress`、`end`、`error`、`cancelled`、`timeout`。

## Provider families

| Family | Request dialects | Stream shape | 默认 host |
| --- | --- | --- | --- |
| `openai` | `openai_chat`, `openai_responses` | `delta_sse`, `semantic_sse` | `api.openai.com` |
| `anthropic` | `anthropic_messages` | `semantic_sse` | `api.anthropic.com` |
| `gemini` | `gemini_generate_content` | `typed_chunk_stream` | `generativelanguage.googleapis.com` |
| `openai_compatible` | `openai_chat` | `delta_sse` | 必须显式 `baseUrl` |
| `openrouter` | `openai_chat`, `stateless_responses` | `delta_sse`, `semantic_sse` | `openrouter.ai` |
| `deepseek` | `openai_chat` | `delta_sse` | `api.deepseek.com` |
| `xai` | `openai_chat`, `openai_responses` | `delta_sse`, `semantic_sse` | `api.x.ai` |
| `fireworks` | `openai_chat`, `fireworks_responses` | `delta_sse`, `semantic_sse` | `api.fireworks.ai` |

OpenAI-compatible 是 adapter family，不是 Yggdrasil 的唯一模型世界观。Anthropic、Gemini 和 Responses 风格 stream 都有不同结构，必须显式适配。

## Profile shape

示例 profile 位于 `examples/model-provider-profiles/`。所有示例使用 `secret_ref`，不包含真实 key。

```json
{
  "family": "anthropic",
  "model": "claude-3-5-sonnet-20241022",
  "credential": "secret_ref:env:ANTHROPIC_API_KEY",
  "baseUrl": "https://api.anthropic.com",
  "headers": {
    "anthropic-version": "2023-06-01"
  }
}
```

规则：

- credential 必须是 `secret_ref:*`、`secretRef:*`、`secret-ref:*` 或 `host:*` reference。
- raw credential、raw-looking header、`api_key` 或 `secret` 字段会被拒绝。
- `invoke` 要求 HTTPS `baseUrl`；`openai_compatible` 没有默认 host，必须显式提供 HTTPS `baseUrl`。
- profile 可以携带 provider-specific `headers` 和 `extra`，但这些字段保持包语义，不进入 kernel ontology。

## 常用能力

### `list_supported_families`

返回八类 family 的默认 base URL、request dialect、stream family、credential header 与注意事项。不会出网。

### `validate_profile`

检查 family、model、credential、base URL、headers，并返回：

- `valid`
- `diagnostics`
- `network_required`
- `secret_refs`
- `network_performed=false`
- `inference_performed=false`

### `normalize_request`

把统一输入转成 provider-specific request shape。示例：

- OpenAI Chat：`/v1/chat/completions`
- OpenAI Responses：`/v1/responses`
- Anthropic：`/v1/messages`
- Gemini：`/v1beta/models/{model}:generateContent`
- OpenRouter：`/chat/completions` 或 `/responses`
- DeepSeek：`/chat/completions`
- xAI：`/v1/chat/completions` 或 `/v1/responses`
- Fireworks：`/chat/completions` 或 `/responses`

### `invoke`

当前不做真实网络调用。它返回 provider-shaped fake/local response 和 `outbound_request_shape`，用于证明 adapter、secret_ref、base URL、method、path 和 redaction shape。

输出必须保持：

```json
{
  "network_performed": false,
  "inference_performed": false,
  "executor_kind": "fake_local",
  "live_call_supported": false
}
```

真实 live call 是后续普通包/host policy 工作，不会通过官方包私有路径获得特权。

### `normalize_stream`

把 provider event/chunk 归一为 `StreamFrameEnvelope` 风格 frames。Provider-specific 字段放进 `payload` 或 `metadata`：

- OpenAI Chat / OpenAI-compatible / OpenRouter / DeepSeek / xAI / Fireworks：delta SSE。
- OpenAI Responses / Anthropic：semantic SSE。
- Gemini：typed chunk stream；把 `GenerateContentResponse` 风格快照适配为 chunk/end/progress。

结构事件（Anthropic `message_start`、`content_block_start`、OpenRouter comment、Gemini 无新增文本的快照）进入 `progress`，不伪装成文本 chunk。Usage、finish_reason、safety ratings、perf metrics、generation id 等进入 `metadata` 或 end payload，不成为 kernel 语义。

### `explain_error`

把 provider-specific code/status 归一为普通错误类别，例如：

- `authentication`
- `permission`
- `rate_limit`
- `billing`
- `not_found`
- `bad_request`
- `tool_schema`
- `timeout`
- `overloaded`
- `stream_error`
- `upstream_malformed`
- `unknown`

这是 package-level 归一化，不是 `kernel.model.error`。

## Manual live call boundary

Alpha 不默认执行真实 provider 请求。后续 live path 必须满足：

1. provider package 声明最小网络权限；
2. caller 或 host 显式授权；
3. secret 通过 host resolver 解析，raw secret 不进入 event/log/audit；
4. request 走 host outbound boundary；
5. audit 只记录 host、method、purpose、secret_refs、usage/cost/error metadata 和 redaction state；
6. stream 统一落到 content-free frame lifecycle；
7. cancel/timeout 不被 provider adapter 私自吞掉；
8. third-party provider package 可以替换官方 package，没有官方优先级。

## 非目标

- 用户余额、充值、计费后台、倍率、渠道管理系统。
- 托管平台代理 key。
- `kernel.model.*`、`kernel.prompt.*`、`kernel.chat.*`、`kernel.embedding.*`。
- 把 OpenAI-compatible 当作唯一模型协议。
- 让官方包绕过 manifest、permission、secret、network 或 audit 边界。

## 验证

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- package check packages/official/model-provider-lab/manifest.yaml
tsc -p clients/web/tsconfig.json --noEmit
```

当前 conformance 包含 `official.model_provider_lab`、`official.model_provider_lab_invoke_core`、`official.model_provider_lab_normalize_stream` 和 outbound fake executor hostile cases。
