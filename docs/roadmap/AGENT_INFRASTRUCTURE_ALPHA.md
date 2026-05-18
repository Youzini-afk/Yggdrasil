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

## J4 — Capability Tool Bridge Lab ✅

- 新增普通 tool bridge 包。
- 发现 capabilities，预览权限，显式 provider selection，通过 `kernel.capability.invoke/stream` 调用。
- hostile conformance 覆盖 ambiguous provider、denied invoke、official no-priority。
- **交付**：
  - `packages/official/capability-tool-bridge-lab/manifest.yaml` — 普通包，6 capabilities（discover_tools、preview_tool_permissions、invoke_tool、stream_tool、explain_tool_call、echo），3 surfaces（forge_panel + assistant_action + home_card），permissions {} 无网络声明。
  - `crates/ygg-runtime/src/inproc/capability_tool_bridge_lab.rs` — inproc handler，返回 deterministic tool-bridge plans（discover_tools 标记 ambiguous/rejected 不偏袒 official；preview_tool_permissions 报告 missing_permissions；invoke_tool/stream_tool 要求显式 provider，ambiguous/missing rejected；explain_tool_call 返回 audit-safe summary；raw secret payload 返回 unsafe_blocked）。
  - `crates/ygg-cli/src/conformance/official_labs.rs` — `capability_tool_bridge_lab()` conformance 用例：验证 load package；discover_tools 对 ambiguous providers 标记 rejected；explicit third-party provider works as plan；official provider not preferred；invoke_tool missing provider rejected；preview denied reports missing permission；raw secret payload unsafe_blocked；surfaces discoverable。
  - Conformance 总数 +1（101 个具名用例）。

## J5 — Forge / Assist 观测面 ✅

- 展示 agent trace、tool timeline、proposal explanation、stream text、audit/redaction badges。
- 仅用 public protocol 和 surface discovery。
- **交付**：
  - `clients/web/src/agent/observability.ts` — 纯 UI helper，用通用字符串启发式从 events/proposals/surfaces/capabilities 中提取 agent-like 观测数据（不 hardcode official 包）。
  - `clients/web/src/surfaces/forge.ts` — 新增 "Agent Observability" section：cards/summary（agent pkg/surf/run/tool/proposal/stream 计数）、trace timeline（最新 package-owned trace/tool/run signals）、tool bridge diagnostics badges（ambiguous/rejected/provider/permission/redaction）、proposal explanation（复用 T4 text preview）。保留现有 JSON inspectors。
  - `clients/web/src/drawer/assistant.ts` — 新增轻量 "Agent Readiness" panel：当前发现的 agent-like surfaces/capabilities count，强调 no real model / no network / proposal-gated / tool bridge plan-only。按钮 disabled，不真正启动 agent。
  - `clients/web/src/main.ts` — 传入辅助 view 数据，wire 到 Forge 与 Assistant Drawer。
  - `clients/web/src/styles.css` — Agent Observability 与 Agent Readiness 样式。
  - `clients/web/README.md` — J5 文档。

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
