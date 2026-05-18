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

## Phase L1 — Host EnvSecretResolver

实现 host-owned env secret resolver：

- 支持 `secret_ref:env:NAME` / `secretRef:env:NAME` / `secret-ref:env:NAME` / `host:env:NAME`。
- 默认 deny-all；必须 host config 显式 allow env names。
- 缺失、未 allow、格式错误都返回 typed error。
- Raw value 只在 host 内部短暂存在，不序列化；audit/error 只出现 secret_ref/env name，不出现 value。
- Conformance 覆盖 allowed/missing/denied/no-leak。

## Phase L2 — LiveHttpOutboundExecutor

实现 host live HTTP executor：

- `reqwest + rustls`。
- 默认不启用；`RuntimeConfig` 必须 opt-in。
- HTTPS-only。
- redirect 默认 reject，或只允许同 host/显式 allowlist。
- connect/request timeout；stream 后续扩 idle watchdog。
- headers/body 只以 shape/audit metadata 记录，不保存 raw auth/body。
- Denied/policy mismatch 时 executor 不被调用。
- Conformance 默认使用 local loopback fixture 或 fake，不依赖公网。

## Phase L3 — Public outbound/secret boundary

公开普通能力包可用的 content-free host boundary：

- `kernel.secret.resolve` 或等价 host protocol method（只返回 redacted/usable-by-host handle，不把 raw key发给包）。
- `kernel.outbound.execute` / `kernel.outbound.stream` 或等价 capability-facing method。
- Official 和 third-party provider 包走同一路径。
- 文档明确 subprocess 任意联网仍不是 OS 级拦截；未受控 subprocess provider 不得作为 live provider 默认形态。

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
