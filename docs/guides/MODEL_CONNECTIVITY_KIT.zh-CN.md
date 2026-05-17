# Model Connectivity Kit

> [English](./MODEL_CONNECTIVITY_KIT.md) · [中文](./MODEL_CONNECTIVITY_KIT.zh-CN.md)

Model Connectivity Kit Alpha 是 Yggdrasil 第一层 model-provider 基础设施。它准备 provider profiles 与 route plans，但不把数据发送给外部 providers。

## 包含的能力包

- `official/model-connector-lab`
  - `describe_families`
  - `validate_profile`
  - `mask_secret`
  - `discovery_plan`
  - `compat_report`
- `official/model-routing-lab`
  - `define_binding`
  - `resolve_binding`
  - `preview_routes`
  - `params_normalize`
  - `compat_report`

## Alpha 安全边界

Alpha 刻意保持 no-network 与 no-inference：

- discovery 是 plan，不是 live provider query；
- validation 是 structural，不是 credential verification；
- 接受 `secret_ref`，拒绝 raw secrets；
- route resolution 从显式 bindings 中选择，不创建隐藏 global route；
- params normalization 保持 provider-specific options namespaced；
- 相关输出包含 provenance 以及 `network_performed: false` 或 `inference_performed: false`。

## 典型流程

1. 用 `official/model-connector-lab/describe_families` 描述 provider families。
2. 用 `official/model-connector-lab/validate_profile` 验证 redaction-safe profile。
3. 用 `official/model-connector-lab/discovery_plan` 生成 no-network model discovery plan。
4. 用 `official/model-routing-lab/define_binding` 定义 consumer-slot bindings。
5. 用 `official/model-routing-lab/resolve_binding` 解析 deterministic routes。
6. 用 `official/model-routing-lab/params_normalize` normalize generation-like params。

持久化仍应通过公开 asset/proposal protocol operations。model labs 不获得特殊写权限。

## TavernHeadless 参考

`integrations/tavern-headless/model-connectivity-map.yaml` 跟踪从 TavernHeadless 研查的 provider/profile/instance 行为。这个 map 只是 reference ledger。Yggdrasil 不创建 `tavern-*` model packages。

## Deferred inference

真实 model calls 属于未来能力包族，可能是 `official/model-inference-lab`。在此之前，Yggdrasil 需要先定义 secret resolution、network permission、request/response audit、streaming/cancel policy、usage accounting、provider errors 与 redaction rules。
