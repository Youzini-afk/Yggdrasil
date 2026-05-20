# Experience-Led Platform Beta Plan

> [English](./EXPERIENCE_LED_PLATFORM_BETA_PLAN.en.md) · [中文](./EXPERIENCE_LED_PLATFORM_BETA_PLAN.md)

本文是 Experience-Led Platform Beta 的执行计划。长期战略见 [`docs/product/EXPERIENCE_LED_PLATFORM_BETA.md`](../product/EXPERIENCE_LED_PLATFORM_BETA.md)。本计划是临时 phase plan：每个阶段完成并收敛到 durable docs 后应删除。

## 总原则

- 从 foundation-first 转向 experience-led：新增 substrate 必须服务真实 playable experience 压力。
- Kernel 继续内容无关：不新增 `kernel.experience.*`、`kernel.world.*`、`kernel.scene.*`、`kernel.character.*`、`kernel.turn.*`、`kernel.agent.*`、`kernel.model.*`、`kernel.memory.*`、`kernel.chat.*`。
- Experience 是 package-owned，不是 kernel-owned。
- 官方 reference experience 必须可被第三方包替换。
- 默认 conformance 不依赖公网；live model smoke 仅显式 opt-in。
- UI/Forge 只能走公开协议、events、surfaces、capabilities、assets、projections、proposals，不能读 runtime internals 或 SQLite。
- 每个 phase 完成后必须验证、commit、push。

## Phase 0 — Strategy lock

目标：提交 Experience-Led Platform Beta 长期设计与同步文档，固定阶段路线。

交付：

- `docs/product/EXPERIENCE_LED_PLATFORM_BETA.md` / `.en.md`。
- README、ALPHA_STATUS、NEXT_STEPS 同步。
- 本计划文档。

验证：markdown local link check、`git diff --check`。

## Experience Beta 0 — Thin Experience Runtime Contract

目标：定义普通 package-owned experience 如何启动、暂停、恢复、checkpoint、fork，并与 Agentic Forge run 关联。

交付：

- TypeScript SDK：`sdk/typescript/experience-runtime`，提供 package-owned experience descriptor、state projection、checkpoint、recovery、play surface subscription、Forge/Assist binding 的类型与自测。
- CLI template：`init-package --template experience-runtime` 或扩展现有 `typescript-experience`，生成 deterministic/no-network experience package 骨架。
- 官方普通 reference 包：`packages/official/experience-runtime-lab`，只做 contract/fixture/capability proof，不定义 world/scene/turn 语义。
- Guide：`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md` / `.en.md`。
- Conformance：experience contract、checkpoint shape、recovery shape、no kernel experience namespace、third-party shape parity 基础用例。

红线：不新增 kernel experience/world/turn 方法；不让官方包获得特殊路由。

## Experience Beta 1 — First Real Playable Vertical Slice

目标：尽早构建一个能连续玩 20–30 分钟的 AI-native reference experience，用它拉动后续底座。

交付：

- 官方普通包：`packages/official/living-sandbox` 或等价 Ygg-native experience 包。
- Package-owned state：通过 opaque events/assets/projections 表达，不进入 kernel ontology。
- Play surface descriptor：可从 Home 启动，可在 Play surface 检查当前 state/projection。
- Assist/Forge loop：用户请求修改，Agentic Forge 生成 plan/candidate/proposal，用户 inspect/approve/reject，支持 fork/compare。
- Deterministic/default path：CI/conformance 无公网。
- Optional live model path：只通过现有 inference/model-provider/outbound policy 显式 opt-in。
- Third-party replacement proof：至少一个关键能力可由 example third-party package 替换。

验证：workspace tests、conformance、playable vertical CLI demo、package checks、composition check、Web TS。

红线：不是 chat shell，不是 Tavern clone，不新增 canonical game runtime。

## Experience Beta 2 — State + Asset Pipeline Alpha（已完成）

目标：只补 First Real Playable Vertical Slice 暴露出的最小 state/asset substrate。

已交付：

- 稳定 content-addressed asset helper，使用 FNV-1a 64-bit hash（`fnv1a64:` 前缀，跨运行确定性，替代不稳定的 DefaultHasher）。
- 标准 Beta 2 元数据约定：`content_address`、`provenance`、`disclosure`、`source_refs`、`derived_refs`、`branch_ref`、`state_snapshot_ref`、`projection_ref`、`proposal_ref`、`inference_ref`、`large_output_policy`。
- `official/asset-lab` 扩展 `content_address` 和 `provenance_graph` 能力。
- `official/projection-lab` 扩展 `state_snapshot` 能力。
- `official/playable-creation-board` 扩展 `preview_state_diff` 和 `describe_asset_provenance` 能力（共 13 个）。
- Asset provenance graph、state snapshot convention、branch-aware diff preview 均已实现。
- Large output 通过 asset ref（已有 capability-tool-bridge-lab 推荐已强化）。
- Package-scoped asset permission proof（origin_package_id 强制，跨包 spoof fail-closed）。
- 新增 9 个 conformance 用例（总计 206 个）。

验证：content address stable、provenance graph、state snapshot convention、state diff preview、playable board metadata、large output asset_ref、package scoped proof — 全部 conformance PASS。

红线：不做完整媒体编辑器，不统一所有 media schema，不把 state ontology 放入 kernel — 全部遵守。

## Experience Beta 3 — Experience Observability

目标：让用户/创作者看懂一次 experience 中发生了什么、为什么失败、成本/延迟在哪里。

交付：

- Observability package/surface pattern：session health、package health、agent run health、proposal causal chain、failure breadcrumbs、cost/latency summary。
- Forge panels：experience health、causal chain、asset provenance、failure breadcrumbs。
- Package-owned observability events/projections，不读 runtime internals。
- Guardrail/audit summary 的公开协议 view。

验证：Web TS、conformance、vertical slice 中展示 observability。

红线：不做 SaaS APM，不读 SQLite。

## Experience Beta 4 — Memory / Knowledge Package Alpha

目标：提供普通包形式的长期记忆与知识，按 vertical slice 真实需求最小实现。

交付：

- `official/memory-lab` 或扩展现有 knowledge/context lab，以 package-owned memory records、retrieval trace、proposal-gated memory update、forget/redaction、branch-aware memory view 为核心。
- SDK/helper：memory record、retrieval trace、correction、redaction metadata。
- Vertical slice 接入：如果需要跨 session/branch 记忆，则使用该包；否则只提供 readiness proof。
- Third-party replacement proof。

验证：memory conformance、raw-secret blocking、branch-aware view、proposal-gated mutation。

红线：不新增 `kernel.memory.*`，不做官方唯一 RAG，不做聊天记忆系统。

## Experience Beta 5 — Creator Loop Beta

目标：一个新创作者不读源码，只靠 docs/template/Forge，一天内做出可玩的 package。

交付：

- 更好的 experience templates、fixture runner UX、reload flow polish。
- Composition diagnostics 针对 experience package set、surface slots、replacement candidates、permissions、state/checkpoint capability。
- Forge authoring workflow：package inventory、experience descriptor preview、fixture controls、diagnostics explainability。
- Walkthrough：从 template 到 playable package。

验证：generated package checks、fixture/reload tests、Web TS、doc links。

红线：不做 marketplace/monetization。

## Experience Beta 6 — Sharing / Distribution Alpha + cleanup

目标：支持可分享、可复现、可导入；删除临时计划并收敛 durable docs。

交付：

- Export/import composition bundle。
- Branch/session bundle manifest。
- Package-set lockfile / compatibility report。
- AI disclosure metadata bundle。
- Read-only shared session / async fork sharing proof（本地/文件级 proof 即可）。
- 删除本计划文档，把结果收敛到 README、ALPHA_STATUS、NEXT_STEPS、guides、product docs。

验证：export/import conformance、compatibility report tests、doc links、workspace tests、Web TS。

红线：不做 marketplace、package signing network、hosted billing。
