# Model Connectivity Kit Alpha

> [English](./MODEL_CONNECTIVITY_KIT_ALPHA.md) · [中文](./MODEL_CONNECTIVITY_KIT_ALPHA.zh-CN.md)

## 目标

Model Connectivity Kit Alpha 引入 Yggdrasil-native 的模型 provider 连接能力包，但不创建 model runtime。它参考 TavernHeadless 的 provider 工作，但不是 Tavern wrapper，也不执行真实 inference。

Alpha 范围是 metadata、validation、redaction-safe profile handling、discovery planning、route planning、compatibility reporting 与 parameter normalization。

## 非目标

- 不发网络请求。
- 不测试 API key。
- 不列出 live models。
- 不做 completions、responses、embeddings 或 streaming。
- Kernel types、kernel events 或 kernel protocol methods 不出现 model/provider 术语。
- assets、events、logs、projections、proposals 或 UI surfaces 中不出现 raw secrets。

## 能力包

### `official/model-connector-lab`

Provider/profile 基础设施：

- provider family descriptors；
- profile validation；
- secret masking；
- discovery plans；
- compatibility reports。

Capabilities：

- `official/model-connector-lab/describe_families`
- `official/model-connector-lab/validate_profile`
- `official/model-connector-lab/mask_secret`
- `official/model-connector-lab/discovery_plan`
- `official/model-connector-lab/compat_report`

首批支持 family metadata：

- `openai`
- `openai-compatible`
- `anthropic`
- `google`
- `deepseek`
- `xai`

Alpha 中所有输出必须把 live state 标为 `not_verified` 或 `planned`。

### `official/model-routing-lab`

Consumer-slot route planning：

- define 与 validate consumer slot descriptors；
- resolve static route bindings；
- preview route candidates；
- normalize generation-like parameters；
- explain compatibility/fallbacks。

Capabilities：

- `official/model-routing-lab/define_binding`
- `official/model-routing-lab/resolve_binding`
- `official/model-routing-lab/preview_routes`
- `official/model-routing-lab/params_normalize`
- `official/model-routing-lab/compat_report`

Consumer slots 是 package-owned labels，例如 `play.primary` 或 `analysis.review`；它们不是 kernel semantics。

## TavernHeadless 参考点

已研查的 TavernHeadless 区域：

- `packages/core/src/llm/provider-registry.ts`
- `packages/core/src/llm/types.ts`
- `packages/core/src/llm/llm-service.ts`
- `apps/api/src/lib/llm-provider-discovery.ts`
- `apps/api/src/routes/llm-profiles.ts`
- `apps/api/src/routes/llm-instances.ts`
- `packages/official-integration-kit/sdk/src/resources/llm-*.ts`

TavernHeadless 行为是参考材料。Yggdrasil 使用 native package model，不创建 `tavern-*` 包名。

## Phase 计划

### Phase A — Reference map and fixtures

在 `integrations/tavern-headless/` 下添加 model connectivity map 与紧凑 fixtures。

交付文件包括 `model-connectivity-map.yaml`、fixture provider profiles 与 route binding fixtures。它们只包含 secret references，不包含真实 keys。

验收：

- reference map 指向 `official/model-connector-lab` 和 `official/model-routing-lab`；
- fixtures 不包含真实 secrets；
- 文档说明 no-network/no-inference Alpha 范围。

### Phase B — `official/model-connector-lab`

添加 manifest、capabilities、in-process deterministic behavior、surfaces 与 conformance。

验收：

- profile validation 拒绝 raw secret 泄漏和 malformed base URLs；
- secret masking 从不返回完整值；
- discovery output 是 plan，不是 live results；
- conformance 覆盖支持的 provider families。

### Phase C — `official/model-routing-lab`

添加 manifest、capabilities、in-process deterministic route planning、surfaces 与 conformance。

验收：

- route resolution deterministic；
- fallbacks explicit；
- params normalization 保持 provider-specific options namespaced；
- route plans 不 invoke inference。

### Phase D — Guide and status polish

添加双语 guide，并更新 README/status/conformance docs。

验收：

- 面向用户的文档解释 connector 与 routing labs 如何组合；
- 文档明确 defer `model-inference-lab`；
- conformance count 准确。

### Phase E — Future inference plan and final validation

记录未来 `model-inference-lab` 的前置条件。

验收：

- 未来 inference 需要 secret resolution、network permission、request/response audit、streaming/cancel policy、usage accounting、redaction 与 provider error taxonomy；
- 最终通过 TypeScript、Rust tests、conformance、package checks 与 doc-link check。
