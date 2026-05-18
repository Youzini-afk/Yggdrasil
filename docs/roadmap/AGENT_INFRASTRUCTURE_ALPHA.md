# Agent Infrastructure Alpha

> [English](./AGENT_INFRASTRUCTURE_ALPHA.en.md) · [中文](./AGENT_INFRASTRUCTURE_ALPHA.md)

这是执行期临时计划。完成后删除本文件，并把结果收敛进 README、`docs/ALPHA_STATUS.md`、`docs/roadmap/NEXT_STEPS.md`、conformance matrix 与 durable guides。

目标：让 Yggdrasil 能托管、约束、观察和替换 agent-like capability packages，同时不把 agent/model/prompt/memory/turn 语义加入 kernel。

## J0 — PI Integration Ledger ✅

- 升级 `docs/architecture/PI_INTEGRATION.md` / `.en.md`。
- 新增 `integrations/pi` ledger、lock 和 capability map。
- 固定 pi 能力吸收分级：adapter-now、package-internal optional、reference-only、deferred、rejected。

## J1 — Ygg Agent Adapter SDK ✅

- 新增 `sdk/typescript/ygg-agent-adapter`。
- 提供 capability-to-tool、tool invoke/stream、proposal helper、trace helper、permission/provider diagnostics。
- 不 import private runtime，不依赖 `pi-coding-agent`。
- **交付**：
  - `sdk/typescript/ygg-agent-adapter/index.ts` — 纯 TS，无外部依赖，含 ProtocolClient interface、CapabilityDescriptor / CapabilityTool / ToolCall / ToolResult / AgentTraceEvent / AgentProposalDraft 等稳定类型；createYggAgentAdapter 工厂；capabilityToTool / createCapabilityTool / invokeCapabilityTool / streamCapabilityTool；createTraceEvent / createProposalDraft；diagnosePermissions / diagnoseProvider / blockRawSecrets；runYggAgentAdapterSelfTest 自测覆盖 tool mapping、ambiguous provider rejection、proposal draft、trace event、stream request、raw secret blocking。
  - `sdk/typescript/ygg-agent-adapter/README.md` / `README.en.md` — 中英文文档。
  - `integrations/pi/capability-map.yaml` — 标注 J1 adapter SDK。

## J2 — Agent Runtime Template ✅

- 新增 `ygg init-package --template agent-runtime`。
- 默认 deterministic/no-network subprocess package。
- 包含 streaming run capability、assistant_action/forge_panel surfaces、proposal-first output、package-owned trace events。
- **交付**：
  - `crates/ygg-cli` — `PackageTemplate::AgentRuntime`、`EffectiveTemplate::AgentRuntime`、manifest 生成（4 capabilities: run streaming、explain-run、draft-proposal、echo；2 surfaces: assistant_action + forge_panel；permissions: {}）。
  - `crates/ygg-cli/src/templates/mod.rs` — `typescript_agent_runtime_template()`；使用 `StreamFrameClient`（secure-execution）与 `createTraceEvent`/`createProposalDraft`/`blockRawSecrets`（ygg-agent-adapter）。
  - `crates/ygg-cli/src/conformance/generated.rs` — `generated_agent_runtime_template()` conformance 用例：验证 4 capabilities、run streaming、assistant_action + forge_panel surfaces、no-network、no raw secrets、无 kernel.agent/model/prompt/memory/turn 文本。
  - Conformance 总数 +1（99 个具名用例）。

## J3 — Official Reference Agent Package ✅

- 新增 `packages/official/pi-agent-runtime-lab` 普通包。
- no-network/faux 默认，不真实调用模型。
- 可 stream run、draft proposal、emit trace。
- 官方包无特权，无特殊路由。
- **交付**：
  - `packages/official/pi-agent-runtime-lab/manifest.yaml` — 普通包，5 capabilities（run streaming、explain_run、draft_proposal、summarize_trace、echo），3 surfaces（assistant_action + forge_panel + home_card），approval_policy fork_then_approve，permissions {} 无网络声明。
  - `crates/ygg-runtime/src/inproc/pi_agent_runtime_lab.rs` — inproc handler，返回 deterministic/no-network/faux payload（pi_agent_run_plan、pi_agent_run_explanation、pi_agent_proposal、pi_agent_trace_summary、pi_agent_echo），provenance 含 provider_package_id。
  - `crates/ygg-cli/src/conformance/official_labs.rs` — `pi_agent_runtime_lab()` conformance 用例：验证 no-inference/no-network、approval-gated proposal、surfaces 可发现、provider_package_id 匹配。
  - Conformance 总数 +1（100 个具名用例）。

## J4 — Capability Tool Bridge Lab

- 新增普通 tool bridge 包。
- 发现 capabilities，预览权限，显式 provider selection，通过 `kernel.capability.invoke/stream` 调用。
- hostile conformance 覆盖 ambiguous provider、denied invoke、official no-priority。

## J5 — Forge / Assist Observability

- 展示 agent trace、tool timeline、proposal explanation、stream text、audit/redaction badges。
- 仅用 public protocol 和 surface discovery。

## J6 — Third-party Replacement Proof

- 新增第三方 agent runtime 示例与 composition replacement。
- 证明 third-party agent 与 official agent 能到达同样 surface/capability/proposal/trace 路径。

## J7 — Durable Docs + Cleanup

- 更新 README、ALPHA_STATUS、NEXT_STEPS、CONFORMANCE_MATRIX、package authoring guide。
- 新增 agent package authoring guide。
- 删除本临时计划。

## 非目标

- 不新增 `kernel.agent.*`、`kernel.model.*`、`kernel.prompt.*`、`kernel.memory.*`、`kernel.turn.*`。
- 不做真实 model inference。
- 不整体嵌入 `pi-coding-agent`。
- 不默认提供 bash/read/write/edit tools。
- 不给 official agent 包任何优先级。
