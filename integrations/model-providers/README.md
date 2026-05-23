# Model Provider Integration Research Ledger

> [English](./README.en.md) · [中文](./README.md)

本目录记录 Model Provider Integration Alpha 的调研边界。目标不是做中转站、计费系统或模型网关，而是为 Yggdrasil 的普通能力包接入真实模型 provider 奠定实现依据。

## 研究对象

- OpenAI Responses / Chat Completions
- Anthropic Messages
- Gemini `generateContent` / `streamGenerateContent`
- OpenAI-compatible providers
- OpenRouter
- DeepSeek
- xAI
- Fireworks
- 参考项目：[new-api](https://github.com/Youzini-afk/new-api)
- 参考项目：[TavernHeadless](https://github.com/Youzini-afk/TavernHeadless)

## 对 Yggdrasil 的结论

- 模型 provider 是普通能力包语义，不是 kernel 语义。
- `OpenAI-compatible` 是一个 adapter family，不是唯一世界观。
- Anthropic 和 Gemini 必须有独立 dialect；不能强行塞进 OpenAI delta shape。
- OpenRouter、DeepSeek、xAI、Fireworks 虽多为 OpenAI-style，但需要 provider preset 和 quirk layer。
- usage/cost 是 package output 与 outbound audit metadata，不是用户余额、计费后台或中转站账本。
- 出网必须走 host-enforced outbound boundary 或等价 fake/local executor；否则 secret/network/audit/redaction 只能靠包自律。
- 默认 conformance 使用 fake executor/local mock，不依赖真实 API key 或外网。
- 手动真实调用必须 opt-in，使用 `secret_ref`、network allowlist、redacted audit，并且不作为 CI/release gate。

## 明确不做

- 不做用户余额、充值、倍率、渠道后台或 admin UI。
- 不托管统一平台代理 API key。
- 不新增 `kernel.v1.model.*`、`kernel.v1.prompt.*`、`kernel.v1.chat.*`、`kernel.v1.embedding.*`。
- 不把 provider profile、模型列表、prompt/messages schema 做进 kernel。
- 不让官方 provider 包获得隐式 network、secret、routing 或 UI 特权。

## 文件

- [`provider-matrix.yaml`](./provider-matrix.yaml)：provider/request/stream/tool/usage/error 差异矩阵。
- [`stream-compatibility.md`](./stream-compatibility.md)：流式事件归一化策略。
- [`error-taxonomy.md`](./error-taxonomy.md)：provider error 归一化建议。
- [`new-api-ledger.md`](./new-api-ledger.md)：`new-api` 可吸收与不可吸收经验。
- [`tavern-headless-ledger.md`](./tavern-headless-ledger.md)：TavernHeadless provider/profile 经验。
