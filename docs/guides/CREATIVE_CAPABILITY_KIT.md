# Creative Capability Kit

> [English](./CREATIVE_CAPABILITY_KIT.en.md) · [中文](./CREATIVE_CAPABILITY_KIT.md)

Creative Capability Kit 是 Yggdrasil 第一次把成熟的 headless creative/RP workflows 抽象成 Yggdrasil-native 通用官方能力包。

TavernHeadless 提供边界案例参考，但这些官方包不是 `tavern-*` wrappers：

- `official/persona-lab` 处理 persona-like structured profiles。
- `official/knowledge-lab` 处理 structured knowledge collections 与 match traces。
- `official/context-lab` 处理 bounded context block assembly 与 budget diagnostics。
- `official/text-transform-lab` 处理 deterministic text transform previews 与 pipeline explanations。

## 规则

- Kernel 不知道 persona、knowledge、prompt、worldbook、chat、character 或 model-call 概念。
- 这些包都是普通 manifest/capability/surface packages。
- 兼容输入格式是 adapters 和 fixtures，不是 canonical Yggdrasil ontology。
- Mutation 必须表达为明确的 asset/projection/proposal plans，而不是隐藏的 package state writes。
- 输出应包含 provenance 与 diagnostics。

## Reference tracking

`integrations/tavern-headless/` 记录已研查的 TavernHeadless commit、capability map 与紧凑 fixtures。TavernHeadless 更新时，用它作为 review ledger。

决策词汇：

- `adapted`：已泛化为 Yggdrasil package。
- `adapter_only`：适合 import/export，但不是 canonical。
- `deferred`：有价值，但暂不属于本 kit。
- `rejected`：明确不继承。

## 典型流程

1. 用 `official/persona-lab/import_profile` import profile-like payload。
2. 用 `official/knowledge-lab/import_collection` import knowledge collection。
3. 用 `official/knowledge-lab/match_entries` match knowledge entries。
4. 用 `official/context-lab/assemble_preview` assemble generic context blocks。
5. 用 `official/text-transform-lab/apply_preview` preview deterministic transforms。
6. 如果需要持久化，通过公开协议创建 approval-gated proposal，写 assets 或 rebuild projections。

这条流程刻意停留在 package 层。第三方包只要暴露兼容 capabilities 和 surfaces，就可以替换任意官方 lab。
