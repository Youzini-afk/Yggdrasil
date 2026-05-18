# Model Provider Integration

> [English](./MODEL_PROVIDER_INTEGRATION.en.md) · [中文](./MODEL_PROVIDER_INTEGRATION.md)

本指南记录 Yggdrasil 的 model provider 接入方式。它不是中转站、不是计费系统、不是 provider 后台，也不是内核模型抽象。模型接入必须作为普通能力包工作，并遵守同一套 manifest、权限、`secret_ref`、outbound audit、stream/cancel 和 conformance 边界。

## 当前交付

Model Provider Integration Alpha 与 Live Model Calls Alpha 已完成：

- `integrations/model-providers/` 保存 provider research ledger、provider matrix、stream compatibility notes 和 error taxonomy。
- `sdk/typescript/model-provider-adapter` 提供纯 TypeScript adapter，用于 provider profile、请求归一化、错误分类和 stream event 解析；它不出网、不做计费、不访问私有 runtime。
- `official/model-provider-lab` 是普通官方能力包，覆盖 OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks 八类 provider family。
- `official/model-provider-lab` 能力包括：`list_supported_families`、`validate_profile`、`normalize_request`、`invoke`、`normalize_stream`、`explain_error`、`echo`。
- `invoke` 保留 fake/local provider adapter path：产出 provider 形状的 response 和可审计 `outbound_request_shape`，`network_performed=false`、`inference_performed=false`、`executor_kind=fake_local`，用于默认 conformance 和 adapter 形状验证。
- Host 侧已有 content-free `OutboundExecutor` boundary，默认 deny-all，并有 fake executor、loopback live HTTP executor 和 hostile conformance；这证明 request shape 可以走 host policy/audit 边界，但不声称 OS 级拦截 subprocess 任意联网。
- `kernel.outbound.execute` 是公开出站协议，ordinary packages 和 official packages 必须走同一路径；package principal 来自 protocol context，不能 spoof 其他 package。
- `EnvSecretResolver` 支持 host-owned `secret_ref:env:NAME` allowlist；raw secret 只在 host 内部短暂存在，不进入 event/log/audit/response。
- `LiveHttpOutboundExecutor` 使用 `reqwest + rustls`，默认关闭；HTTPS-only，redirect fail-closed，timeout 必须配置，response/audit 只保留 redacted shape。Loopback conformance 用 `allow_insecure_loopback_for_tests=true`，不依赖公网。
- `secret_headers` 支持 host-side header 注入（例如 Authorization bearer、x-api-key、x-goog-api-key）；缺失/无效 secret fail-closed。`static_headers` 只允许少量非 secret provider/version/format headers（anthropic-version、content-type、accept、http-referer、x-title），并阻止 Authorization/x-api-key/Cookie 等 secret-bearing 或 host-owned headers。
- Live-call conformance 覆盖 DeepSeek canary、OpenAI Chat/Responses、Anthropic Messages、Gemini generateContent、OpenRouter、xAI、Fireworks loopback shapes，以及缺失 secret fail-closed 与无 raw secret 泄漏。
- `normalize_stream` 将 delta SSE、semantic SSE、typed chunk stream 归一为 `StreamFrameEnvelope` 风格 frames：`start`、`chunk`、`progress`、`end`、`error`、`cancelled`、`timeout`；并覆盖 DeepSeek reasoning/cache usage、OpenRouter mid-stream error、xAI reasoning usage、Fireworks perf usage 等 provider quirks。

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

`official/model-provider-lab/invoke` 本身仍然是 fake/local adapter path。真实网络调用不通过官方包私有 runtime access；它必须由 ordinary package 使用公开 `kernel.outbound.execute`，由 host policy、secret resolver 和 outbound executor 控制。

输出必须保持：

```json
{
  "network_performed": false,
  "inference_performed": false,
  "executor_kind": "fake_local",
  "live_call_supported": false
}
```

这保留了 adapter 自测与默认 conformance 的确定性，同时避免官方 provider 包获得第三方包没有的私有出站特权。

### `kernel.outbound.execute`

普通能力包的 live HTTP path：

```json
{
  "capability_id": "example/provider/fetch",
  "destination_host": "api.openai.com",
  "method": "POST",
  "path": "/v1/chat/completions",
  "secret_headers": {
    "Authorization": {"secret_ref": "secret_ref:env:OPENAI_API_KEY", "scheme": "bearer"}
  },
  "body_shape": {"model": "gpt-4o", "messages": [{"role": "user", "content": "hello"}]}
}
```

规则：

- caller package 来自 `ProtocolContext::Package`，不能由 params 伪造；`capability_id` 必须属于 caller package namespace。
- host 先解析 `secret_headers`，解析失败则 fail-closed，不发送请求。
- policy/audit request 与 executor request 的 package/capability/host/method/secret_refs 必须一致，否则 fail-closed。
- raw secret、Authorization value、request body 和 response body 不进 audit；只记录 redacted shape、secret_refs、host、method、purpose、status、usage/cost/error metadata。
- 默认 runtime 仍是 deny-all；live executor 需要 host 显式 opt-in。

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

Alpha 默认 conformance 不依赖公网；manual/live provider 请求必须满足：

1. provider package 声明最小网络权限；
2. caller 或 host 显式授权；
3. secret 通过 host resolver 解析，raw secret 不进入 event/log/audit；
4. request 走公开 `kernel.outbound.execute` 与 host outbound boundary；
5. audit 只记录 host、method、purpose、secret_refs、usage/cost/error metadata 和 redaction state；
6. stream 统一落到 content-free frame lifecycle；
7. cancel/timeout 不被 provider adapter 私自吞掉；
8. third-party provider package 可以替换官方 package，没有官方优先级。

可选真实 DeepSeek smoke path 只在同时满足以下条件时运行：

```bash
YGG_LIVE_MODEL_TESTS=1 DEEPSEEK_API_KEY=... cargo run -p ygg-cli -- conformance
```

默认 CI / 默认 conformance 不会访问公网。

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

当前 conformance 包含 `official.model_provider_lab`、`official.model_provider_lab_invoke_core`、`official.model_provider_lab_normalize_stream`、public `kernel.outbound.execute`、secret header injection、live loopback provider shapes、provider quirk fixtures 和 outbound hostile cases，共 145 个具名 CLI 用例。
