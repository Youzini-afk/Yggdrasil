# Creative Capability Kit Alpha

> [English](./CREATIVE_CAPABILITY_KIT_ALPHA.md) · [中文](./CREATIVE_CAPABILITY_KIT_ALPHA.zh-CN.md)

## 目标

Creative Capability Kit Alpha 把成熟的 headless RP/tooling 思路转化为 Yggdrasil-native、通用的官方能力包。TavernHeadless 是参考语料和测试 oracle，不是 Yggdrasil 的 ontology。

结果必须服务 AI 游戏、RP、互动小说、模拟、世界构建、agents 与外部引擎。它不能创建 `tavern-*` 包，也不能把 prompt/persona/knowledge 概念放进 kernel。

## 不可妥协的边界

- 除非发现通用 package/capability/surface 机制的 bug，否则 kernel 影响为零。
- Kernel type、event 或 protocol method 不得出现 `persona`、`character`、`prompt`、`context`、`worldbook`、`lorebook`、`chat`、`message`、`turn`、`Tavern` 或 model calls。
- 官方包是普通包：相同 manifest、相同 capability routing、相同权限、相同 surface descriptors。
- import/export 兼容是次级层。核心接口即使没有 SillyTavern 或 TavernHeadless 也必须有用。
- 所有生成的 plans 在暗示 mutation 时必须包含 provenance、diagnostics 和 approval-gated proposal shapes。

## 参考来源

已研查的 TavernHeadless 能力：

- character card parsing/export 与 unknown-field preservation；
- worldbook/lorebook normalization、trigger logic、recursion 与 outlet placement；
- preset parsing、prompt-order semantics、compat assembly、native prompt graph compilation；
- regex profile parsing 与 deterministic transform traces；
- prompt runtime traces、source selection、budget pruning 与 template rendering；
- public SDK/OpenAPI boundaries 与 CI/version-check discipline。

这些思路会被改造成通用包：

- `official/persona-lab`
- `official/knowledge-lab`
- `official/context-lab`
- `official/text-transform-lab`

## 标准化能力包目标

### `official/persona-lab`

目标：import、normalize、describe、validate 和 render persona-like structured profiles，不假设 chat characters。

Capabilities：

- `official/persona-lab/import_profile`
- `official/persona-lab/normalize_profile`
- `official/persona-lab/describe_profile`
- `official/persona-lab/render_fragment`
- `official/persona-lab/compat_report`

输出 kind 示例：`persona_profile`、`persona_fragment`、`persona_compat_report`。

### `official/knowledge-lab`

目标：管理 structured knowledge collections 与 deterministic activation/matching plans，不让 lorebook/worldbook 语义成为 canonical。

Capabilities：

- `official/knowledge-lab/import_collection`
- `official/knowledge-lab/normalize_entries`
- `official/knowledge-lab/match_entries`
- `official/knowledge-lab/injection_plan`
- `official/knowledge-lab/compat_report`

输出 kind 示例：`knowledge_collection`、`knowledge_match_result`、`knowledge_injection_plan`。

### `official/context-lab`

目标：从显式 sources、budgets 与 policies 组装 bounded context blocks，服务任意下游 consumer。

Capabilities：

- `official/context-lab/assemble_preview`
- `official/context-lab/inspect_layers`
- `official/context-lab/budget_plan`
- `official/context-lab/render_template`
- `official/context-lab/explain_assembly`

输出 kind 示例：`context_preview`、`context_layer_inspection`、`context_budget_plan`。

### `official/text-transform-lab`

目标：deterministic text transforms、templates、regex-like rules、macro imports、pipeline explanations 与 compatibility diagnostics。

Capabilities：

- `official/text-transform-lab/import_rules`
- `official/text-transform-lab/validate_rules`
- `official/text-transform-lab/apply_preview`
- `official/text-transform-lab/explain_pipeline`
- `official/text-transform-lab/compat_report`

输出 kind 示例：`text_transform_profile`、`text_transform_preview`、`text_transform_pipeline`。

## Upstream tracking

创建 `integrations/tavern-headless/` 作为参考 ledger，而不是 runtime dependency：

- `upstream.lock.toml`：已研查 path/ref/version/date/toolchain。
- `capability-map.yaml`：把 TavernHeadless subsystems 映射到 Yggdrasil-native packages，状态为 `adapted|deferred|adapter_only|rejected`。
- `README.md`：说明 TavernHeadless 是参考来源。
- `fixtures/`：character cards、knowledge books、presets/context、text transform rules 的紧凑示例。

未来更新检查应比较已研查 TavernHeadless commit 和变化的 subsystem paths，然后决定 adopt、adapt、defer 或 reject。

## Phase 计划

### Phase A — Reference tracking and fixtures

添加 integration ledger 与紧凑 fixtures。补充文档解释从 TavernHeadless 到 Yggdrasil-native packages 的抽象路径。

交付文件位于 `integrations/tavern-headless/`，包括 `upstream.lock.toml`、`capability-map.yaml`、参考 README，以及 persona、knowledge、context 与 text transforms 的紧凑 fixtures。

验收：

- 没有 product package 命名为 `tavern-*`；
- fixtures 小且位于本仓库；
- reference map 指向 Yggdrasil package 名；
- 文档说明兼容输入不是 canonical ontology。

### Phase B — `official/persona-lab`

添加普通 manifest、capabilities、surfaces、in-process behavior、conformance、Forge visibility 与文档。

验收：

- 能把 profile/card-like payload import 成 normalized profile output；
- diagnostics 保留 unknown fields；
- render fragment 包含 provenance；
- 不直接 mutation asset。

### Phase C — `official/knowledge-lab`

添加 ordinary package，用于 knowledge collections、entry normalization、matching 与 injection planning。

验收：

- deterministic keyword matching 带 trace；
- injection plan 仍是 plan，不做隐式 context mutation；
- compatibility report 能描述 worldbook-like inputs，但不让它成为 canonical。

### Phase D — `official/context-lab`

添加普通 package，用于 context previews、layer inspection、template rendering 与 budget planning。

验收：

- 输出使用 generic context blocks，不是 chat messages；
- included 与 omitted sources 都有 reasons；
- budget accounting 可见；
- 不做 model calls。

### Phase E — `official/text-transform-lab` and guide polish

添加普通 package，用于 transform rule import/validation/preview 与 pipeline explanation。更新双语 guide、README、status、conformance matrix，并按需调整 UI surface 文案。

验收：

- deterministic transform preview 包含 trace；
- unsafe/unsupported rules 产生 diagnostics；
- conformance 为四个包增长；
- TypeScript、Rust tests、conformance、package checks 与 doc links 全部通过。

## Validation gate

每个 phase 必须通过对应 scoped checks 后再 commit/push。最终 gate：

```bash
tsc -p clients/web/tsconfig.json --noEmit
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

每个新官方包都必须运行 package check。
