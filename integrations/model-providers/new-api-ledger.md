# new-api Ledger

> 中文默认说明。本文件总结 `/workspace/nashiyard/new-api` 对 Yggdrasil 有用和不应吸收的经验。

## 可吸收经验

- Adapter 分层：provider adapter 负责 URL、headers、request conversion、response conversion、stream handling。
- Runtime context bus：调用过程中携带 provider/channel/model/usage/header/stream metadata，便于观测。
- Request conversion chain：chat ↔ responses、provider-specific normalization 应留下转换链路。
- Stream scanner：统一 SSE scanner、heartbeat、done/stop/error 语义、客户端断开处理。
- Model mapping：链式映射、cycle detection、provider-specific suffix/compact/thinking policy。
- Header/base URL quirks：header override、auth placeholder、client header passthrough、Cloudflare/Vertex/Azure path quirks。
- Usage metadata：prompt/completion/total、cache tokens、reasoning tokens、usage source、stream status。
- Error wrapping：统一解析上游非 2xx 和 provider-specific error body，保留 status/provider code，mask raw sensitive data。

## 不吸收

- 用户余额、充值、倍率、pre-consume/refund、subscription。
- Admin/channel 管理 UI。
- 自动禁用/启用 channel 的运营治理。
- 托管平台代理 API key 或统一 relay endpoint。
- 把 channel/provider ontology 放进 Ygg kernel。

## 对 Yggdrasil 的实现启发

- 多 provider 接入应分为 transport layer、canonical model layer、provider quirk layer。
- OpenAI-compatible 仍需要 provider presets，不应只有一个 generic OpenAI mode。
- Usage/cost 只作为 package output/audit metadata，不做计费系统。
- Base URL 和 redirect 必须走 host policy 检查。
