# TavernHeadless Provider Ledger

> 中文默认说明。本文件总结 `/workspace/Yggdrasil/TavernHeadless` provider/profile 经验。

## 可吸收经验

- Provider profile 是可激活、可 fallback、可 masking 的配置对象，而不是 kernel state。
- Routing 按 provider type 选择 adapter；OpenAI/DeepSeek/xAI/openai-compatible 可共享 OpenAI-style 工厂，Anthropic/Gemini 需要独立 adapter。
- Request normalization 应留在 package/SDK 层：generation params、history normalization、assistant prefill、token budget 都是产品语义。
- Streaming 有两层：provider stream parser 与 UI reducer/tool-event grouper。Kernel 只需要通用 stream frame lifecycle。
- Discovery/hello probe 是 package 层行为，不是 kernel provider registry。

## 不进 kernel

- provider 类型、API key、base URL、profile encryption/masking。
- model discovery、hello probe、provider-specific response parsing。
- slot routing、session/global fallback、active profile resolution。
- prompt/conversation history normalization、assistant prefill strategy。
- tool event grouping、replay safety hints。

## 对 Yggdrasil 的映射

- `model-connector-lab` 保持 profile/readiness/discovery-plan 边界。
- `model-routing-lab` 保持 route planning/binding 边界。
- 新 `model-provider-lab` 承担 request normalization、fake/local invoke、stream normalization、error mapping。
- 真实 provider profile、prompt/messages shape 仍为 package schema，不进入 kernel。
