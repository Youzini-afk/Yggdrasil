# Live Model Calls Alpha

> [English](./LIVE_MODEL_CALLS_ALPHA.en.md) · [中文](./LIVE_MODEL_CALLS_ALPHA.md)

## 目标

把 Model Provider Integration Alpha 的 fake/local provider path 推进为真实 live call path：

```text
secret_ref → host secret resolver → public outbound boundary → live HTTPS executor → provider adapter → normalized response/stream → redacted audit
```

这不是中转站、计费系统、渠道后台或 kernel model ontology。所有 live model calls 仍然是普通能力包行为，必须保持官方包与第三方包同权。

## 不变量

- 不新增 `kernel.model.*`、`kernel.prompt.*`、`kernel.chat.*`、`kernel.embedding.*`。
- Provider 包不得直接读取 env、`.env`、credential store 或 raw key。
- Provider 包不得直接使用 `reqwest`/`fetch`/`curl`/provider SDK 绕过 host outbound boundary。
- Official provider 包不得走 private runtime network API；第三方包必须能使用同一公开边界。
- 默认 CI/conformance 不依赖外网或真实 key；live tests 必须 opt-in。
- Raw secret 不得进入 event、audit、log、error、stream frame、fixture 或 doc example。
- Live executor 默认关闭；host profile 必须显式允许 env secret 和 outbound host/method。
- 真实 HTTPS 只能通过 host-controlled executor，且必须 enforce timeout、redirect policy、redaction 和 audit。

## Phase L0 — 契约冻结（本文件）

交付：

- 本临时计划。
- Live-call contract：secret resolver、public outbound、redacted audit、live test opt-in、provider 不直连。
- Roadmap/status 指向 Live Model Calls Alpha。

验证：

- 文档链接检查。
- `kernel.model/prompt/chat/embedding` 只出现在非目标/禁止项说明。

## Phase L1 — Host EnvSecretResolver ✅

已实现 host-owned env secret resolver：

- 支持 `secret_ref:env:NAME` / `secretRef:env:NAME` / `secret-ref:env:NAME` / `host:env:NAME`。
- 默认 deny-all；host config 必须显式 allow env names（allowlist-only）。
- 缺失、未 allow、格式错误都返回 typed error。
- Raw value 只在 host 内部短暂存在，不序列化；audit/error 只出现 secret_ref/env name，不出现 value。
- `Runtime::resolve_secret_ref` host 内部方法，用于 host 在能力调用时解析 secret。
- `extract_env_name` 辅助函数只识别 `env` vault；`host:<key>`（不含 `env:` 前缀）不被视为 env 引用。
- Conformance 覆盖 allowed/missing/denied/no-leak（3 个新用例：`secret.env_resolver_allowed`、`secret.env_resolver_denied`、`secret.env_resolver_missing_no_leak`）。

## Phase L2 — LiveHttpOutboundExecutor ✅

已实现 host live HTTP executor：

- `reqwest + rustls`，不启用 native-tls。
- 默认不启用；`RuntimeConfig` 必须 opt-in（`OutboundExecutorConfig::LiveHttp(config)`）。
- HTTPS-only；非 HTTPS URL 被 reject（fail-closed）。
- Redirect 默认 reject（`allow_redirects: false`）；L2 不实现 redirect following，且 `allow_redirects=true` 会 fail-closed，直到 redirect target policy re-check 落地。
- Connect/request timeout 可配置；stream idle watchdog 延后至 stream phase。
- Headers/body 只以 shape/audit metadata 记录，不保存 raw auth/body。
  - 只发送 `content-type: application/json` 和 `x-ygg-outbound` placeholder headers。
  - L2 不注入 secret（L3 负责通过 host boundary 注入）。
  - 响应 headers_shape 只记录 content-type 和 request-id 安全 headers 值；其余 header 值替换为 `[redacted]`。
  - 响应 body_shape：JSON 内 secret 字段替换为 `[redacted]`；非 JSON 只记录 kind/bytes_captured。
- Denied/policy mismatch 时 executor 不被调用。
- 错误归一为 status="error" 或 "timeout"，不包含 raw body/secret。
- `allow_insecure_loopback_for_tests` 默认 false；仅允许 127.0.0.1/localhost 的 http:// URL，用于 conformance 测试。
- `LiveHttpOutboundExecutorConfig` 提供 timeout_ms、connect_timeout_ms、allow_redirects、max_response_preview_bytes、allow_insecure_loopback_for_tests。
- Conformance 3 个新用例：`outbound.live_http_default_disabled`、`outbound.live_http_rejects_insecure_url`、`outbound.live_http_redacted_shape`。不依赖公网。

## Phase L3 — Public outbound/secret boundary ✅

公开普通能力包可用的 content-free host boundary：

- `kernel.outbound.execute` 公开协议方法：允许 ordinary package 通过 host outbound executor 发起出站请求。Params 接受 capability_id、destination_host、method、path、secret_refs、metadata、body_shape。package_id 从 ProtocolContext principal 强制确定——调用者不能在 params 中 spoof 不同的 package_id（host_dev/host_admin principal 可在 params 中指定用于测试）。Dispatch 调用 `execute_outbound_with_policy`，response 经过额外 raw-secret 防护 sweep。不新增 `kernel.secret.resolve`（raw secret 不返回给包）。L3 不注入 secret headers（真实注入延后至 L4/L5）。
- Official 和 third-party provider 包走同一路径。
- 文档明确 subprocess 任意联网仍不是 OS 级拦截；未受控 subprocess provider 不得作为 live provider 默认形态。
- Conformance 新增 4 个用例：`outbound.execute_package_allowed`、`outbound.execute_spoofed_package_id_rejected`、`outbound.execute_no_permission_denied`、`outbound.execute_no_raw_secret_in_response`。不依赖公网。

## Phase L4 — First live provider canary

先跑通一个 provider 的真实 invoke + stream，优先 DeepSeek / OpenAI-compatible：

- env secret opt-in。
- live invoke。
- live stream。
- auth failure、timeout、rate limit/bad request 归一。
- stream cancel/timeout 通过 host boundary。
- `conformance live-model` 手动 opt-in；默认 conformance 仍本地稳定。

## Phase L5 — OpenAI / Anthropic / Gemini live adapters

扩展三个代表性非同构 API：

- OpenAI Chat/Responses。
- Anthropic Messages named SSE。
- Gemini generateContent / streamGenerateContent。

## Phase L6 — OpenRouter / xAI / Fireworks / DeepSeek quirks

补齐剩余 provider family：

- OpenRouter comments + mid-stream error。
- DeepSeek reasoning_content / final usage chunk / keep-alive。
- xAI reasoning timeout / chat vs responses。
- Fireworks responses-style stream fixture。

## Phase L7 — Durable docs + cleanup

收敛为长期文档：

- `docs/guides/LIVE_MODEL_CALLS.md`。
- live setup examples（只含 `secret_ref`，不含 raw key）。
- 更新 README / ALPHA_STATUS / NEXT_STEPS / CONFORMANCE_MATRIX。
- 删除本临时计划。

## 验收标准

Alpha 完成时必须证明：

1. 至少一个 provider 可以真实 invoke。
2. 至少一个 provider 可以真实 stream。
3. 所有 live 请求经过 host outbound executor。
4. 未授权 host/method 被拒绝。
5. Provider 包不能直接读取 env secret。
6. EnvSecretResolver/HostSecretResolver 可以注入 key，但 raw value 不泄漏。
7. audit event 覆盖调用生命周期且脱敏。
8. live conformance opt-in；默认 conformance 仍离线通过。
9. 官方 provider 无 private outbound 特权，第三方 provider 可走同一路径。
