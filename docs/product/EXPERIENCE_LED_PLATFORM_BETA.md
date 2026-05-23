# Experience-Led Platform Beta

> [English](./EXPERIENCE_LED_PLATFORM_BETA.en.md) · [中文](./EXPERIENCE_LED_PLATFORM_BETA.md)

本文档固定 Agentic Forge 之后的长期方向：Yggdrasil 应收敛 foundation-first 工作。下一阶段应由真实 AI-native playable experience 牵引剩余底座建设。

它不是临时计划，也不是某个官方体验的设计文档。它是一份产品-基础设施战略。它说明为什么当前底座已经足以进入体验牵引阶段，哪些基础设施必须由真实体验压测后补强，哪些方向应明确延后，以及 Yggdrasil 如何避免退化成聊天壳、API gateway、传统插件 host 或普通 agent 框架。

## 结论

Yggdrasil 的基础设施不是生产级完整平台，但已经完成了停止 foundation-first 所需的部分。

已经站稳的部分包括：

- 内容无关内核。
- 清单驱动的能力包。
- 官方包无特权与第三方替换证明。
- 公开协议、HTTP `/rpc`、SSE 与 host stdio。
- opaque events、SQLite rehydrate、资产、分支、projection、提案。
- 身份、权限 grants、`secret_ref`、outbound executor、脱敏审计、流式/取消生命周期。
- cloud API adapter packages、live model calls、transport-neutral inference seam。
- Agentic Forge：package-owned run lifecycle、plan graph、scratch candidates、inference nodes、tool bridge v2、Forge control-room shell、third-party replacement proof。
- authoring/composition tooling 与 conformance。

因此，下一阶段的问题不再是：

```text
还能不能再补一个抽象层？
```

而是：

```text
现有底座能不能承载一个真正可玩、可改、可 fork、可持续的 AI-native experience？
```

如果这个问题没有被真实体验回答，继续扩基础设施会变成虚假进展。

## 外部参考给出的信号（2026-05-20）

本节记录近期外部参考对 Yggdrasil 路线的启发。它们不是要被复制的产品形态，也不是 Yggdrasil 的依赖或承诺；它们只是用来校准 AI 游戏平台真正需要哪些底座。来源链接用于后续复核，若外部产品路线变化，应重新审查这些信号。

### Roblox Cube / 4D generation：创作对象要能进入运行时

Roblox 2026 的 Cube / 4D generation 方向给出的信号是：AI 生成不再只是静态 3D 资产。它开始面向可交互对象。schema 将对象拆成必要部件，生成 geometry 后再 retarget 行为脚本，让对象能被玩家实际使用。参考：Roblox, [Accelerating Creation, Powered by Roblox’s Cube Foundation Model](https://about.roblox.com/newsroom/2026/02/accelerating-creation-powered-roblox-cube-foundation-model)；Roblox, [Accelerating AI Inference for 3D Creation on Roblox](https://about.roblox.com/newsroom/2025/06/accelerating-ai-inference-roblox-3d-creation)。

对 Yggdrasil 的含义：

- 资产不能长期只是 opaque put/get/list。
- 生成物需要 provenance、derived asset relationship、behavior binding、preview、diff 与 runtime attachment metadata。
- 玩家在体验内创作本身可以成为玩法，而不是创作者工具的附属功能。

Yggdrasil 不应内置车、门、角色、场景等语义。但它应提供足够强的资产、状态、提案和分支底座，让普通包能生成并审查“可进入运行时”的对象。

### Roblox Studio agentic：主线是 plan / build / test loop

Roblox Studio 2026 的 agentic 方向给出的信号是：成熟的游戏创作 agent 主线不是聊天。它围绕 Planning Mode、可审查和可编辑的行动计划、结构化任务清单、parallel agents、playtesting agent、日志和数据模型分析、自我修正 loop，以及通过 unprivileged APIs / MCP 暴露上下文给第三方工具。参考：Roblox, [Roblox Studio is Going Agentic](https://about.roblox.com/newsroom/2026/04/roblox-studio-going-agentic)。

对 Yggdrasil 的含义：

- Agentic Forge 的方向正确：agent 不应是聊天框，而应是围绕创作任务的计划、候选、工具、测试、审查和提案系统。
- 下一步不能停留在可展示 plan graph；必须让 agent 围绕真实 experience 做 plan/build/test。
- Playtesting agent 的价值在于读取状态、日志、事件、projection 与用户目标，产生可审查修复，而不是自动越权修改。

Yggdrasil 的 agent 仍应是普通 package-owned creative process，不能变成 `kernel.v1.agent.*`。

### Roblox Hybrid Architecture：不要把 neural world model 当游戏引擎

Roblox 2026 hybrid architecture 对我们有用的关键判断是：Video World Models 可以产生视觉梦境，但缺少持久状态、一致逻辑、长期记忆、用户输入控制和真正的多人模拟。Roblox 的方向是让 Game Engine 维护共享一致状态、符号逻辑和可重复模拟，让 Video World Model 负责随机视觉。参考：Roblox, [Introducing the Roblox Hybrid Architecture](https://about.roblox.com/newsroom/2026/04/roblox-reality-hybrid-architecture-democratizing-photorealistic-multiplayer-gaming)。

对 Yggdrasil 的含义：

```text
AI 可以梦；平台必须记得、验证、分叉、恢复。
```

Yggdrasil 不应追逐“纯模型运行的游戏世界”。它需要的是：

- package-owned state convention。
- event-sourced state mutation。
- snapshot/checkpoint assets。
- projection-backed inspection。
- replay where possible。
- inference provenance。
- branch-aware state diff。

这些是状态底座，不是 world ontology。它们不应把 world、scene、character、turn 放进内核。

### Unity AI：AI 必须嵌入创作上下文，并提供数据控制

Unity AI principles 给出的信号是：AI 工具必须嵌入创作上下文，并提供数据控制、usage visibility、provider labels 和 generated asset metadata。Unity 把 AI 放进 Editor 上下文，关注项目结构、GameObjects、prefabs、render pipeline、console error resolution、generated asset metadata、usage reporting、provider labels、data ownership、training opt-in off by default、local Sentis inference。参考：Unity, [Unity AI Guiding Principles](https://unity.com/legal/unityai-guiding-principles)；Unity, [2026 Unity Game Development Report](https://unity.com/blog/2026-unity-game-development-report-trends)。

对 Yggdrasil 的含义：

- Forge 不能只是一个 trace viewer；它必须成为创作上下文的控制室。
- 重要上下文包括会话、分支、projection、资产图、package set、提案队列、agent run、tool/inference trace、failure state 和 cost/latency。
- 生成资产需要可追踪 metadata，便于搜索、审计、删除、披露和合规。
- 数据控制与可替换模型路径应保持 host-owned / package-owned，而不是平台托管 key 或统一 provider 后台。

### Inworld Agent Runtime：graph、stream、memory、knowledge、safety、telemetry 是成熟 AI runtime 的基本部件

Inworld Runtime 给出的信号是：成熟 AI runtime 往往围绕 graph、nodes、edges 和 execution stream 组织，并包含 MemoryUpdate、MemoryRetrieve、Knowledge、Safety、Intent、GoalAdvancement、STT/TTS、Telemetry、MCP tool nodes 等部件。参考：Inworld, [Graphs](https://docs.inworld.ai/node/core-concepts/graphs)；Inworld, [Unity Runtime Reference Overview](https://docs.inworld.ai/Unity/runtime/runtime-reference/overview)。

对 Yggdrasil 的含义：

- Agentic Forge 的 plan graph 与 stream/cancel/tool bridge 是正确基础。
- 但 AI-native experience 会很快需要普通包形式的 memory、knowledge、goal/progress、safety、telemetry 能力。
- 这些都不应进入内核；它们应通过 package-owned events/assets/projections/proposals 暴露。

### Steamworks / GDC：AI 游戏需要披露、guardrails、成本和信任策略

Steamworks Content Survey 将 AI 内容区分为 Pre-Generated 与 Live-Generated。Live-Generated AI 要求说明 guardrails，确保不会生成非法内容。如果实时 AI 服务带来持续成本，开发者需要管理访问和玩家支付方式。GDC 2026 State of the Game Industry 的公开摘要显示 GenAI 已被相当比例从业者使用，但行业负面情绪仍很高。参考：Steamworks, [Content Survey](https://partner.steamgames.com/doc/gettingstarted/contentsurvey)；Business Wire / GDC, [2026 State of the Game Industry Report announcement](https://www.businesswire.com/news/home/20260129438528/en/2026-State-of-the-Game-Industry-Report-Reveals-Widening-Effect-of-Layoffs-Broader-Perspectives-on-Generative-AI-Unionization-Tariffs-and-More)。

对 Yggdrasil 的含义：

- AI-generated content 必须有 provenance、metadata、guardrail/audit trail、rights/licensing/disclosure metadata。
- Live AI 体验需要 cost/usage visibility、policy hooks、脱敏、report/export logs。
- 用户与创作者应能看懂“AI 做了什么、为什么、使用了哪些来源、可能有什么风险”。

Yggdrasil 当前的 proposal、approval、audit、redaction、branch、Forge control-room 是优势，但还需要 asset-level disclosure 与 experience-level observability。

## 当前阶段判断

### 已足够停止 foundation-first 的部分

以下内容已经足以支撑真实 vertical slice：

- package 加载、能力调用、provider 显式选择、official no privilege。
- 公开协议与 Web shell 基础面。
- event/session/branch/proposal/projection 的最小游创底座。
- secure execution 与 live model opt-in。
- transport-neutral inference 与 cloud adapter 降级定位。
- Agentic Forge 的 branch-aware candidate / compare / promote / tool / inference scaffold。
- authoring/composition/conformance。

因此后续不应再以“补齐一切底座”为主线。

### 仍薄弱但需要由体验牵引的部分

这些内容重要，但应服务第一个真实 playable experience，而不是作为独立大抽象先行：

- Experience runtime contract。
- Package-owned state / snapshot / checkpoint / replay pattern。
- Asset pipeline：content-addressed blobs、provenance graph、derived assets、AI disclosure metadata。
- Memory / knowledge package pattern。
- Experience observability：health、latency、cost、failure breadcrumbs、causal chain。
- Compatibility / migration：package data migration、asset metadata versioning、projection rebuild policy、composition upgrade report。
- Sharing / distribution primitives：composition export/import、branch/session bundle、package-set lockfile。

### 明确延后

以下方向有价值，但不应抢在真实 experience proof 前面：

- Marketplace、creator economy、rating/review、revenue split。
- SaaS billing、user balance、provider key hosting、channel/admin backend。
- 完整 auth/tenant/cloud product。
- 完整实时多人游戏服务器、authoritative multiplayer、co-presence conflict resolution。
- 本地大模型管理器、权重下载器、GPU scheduler。
- 中央内容审查系统。
- 官方 world/scene/character/director runtime。

## Experience-Led Platform Beta 的原则

### 1. 新底座必须被真实体验牵引

新增底座必须回答一个真实 experience 的压力：

- 玩 20–30 分钟时状态如何保持？
- 模型失败时如何恢复？
- 用户如何看懂 agent 的改动？
- 生成 asset 如何被追踪、预览、删除、披露？
- 分支之间如何比较？
- package 升级后旧 session 如何迁移？

如果新增能力不能服务这些问题，应延后。

### 2. 内核继续保持内容无关

严禁新增：

```text
kernel.v1.agent.*
kernel.v1.model.*
kernel.v1.prompt.*
kernel.v1.memory.*
kernel.v1.world.*
kernel.v1.scene.*
kernel.v1.character.*
kernel.v1.turn.*
kernel.v1.chat.*
```

可以新增或强化的是内容无关平台机制，例如 asset blob、resource policy、projection execution、event subscription permission、transport parity、health/audit records。即便如此，也应优先检查是否可由 package 协议和现有底座表达。

### 3. Experience 是 package-owned，不是 kernel-owned

Yggdrasil 可以定义 experience package pattern、surface contract、state snapshot convention、checkpoint asset convention，但不能把题材或玩法语义纳入内核。

一个 reference experience 可以是官方包，但必须可被第三方包替换。官方 reference 的职责是压测底座，不是定义唯一正统玩法。

### 4. Agent 是创作协作者，不是特权执行者

Agentic Forge 的下一步是围绕真实 experience 做 plan/build/test：

- 观察 session/branch/projection/assets/events。
- 在 scratch branch 里探索。
- 生成 candidate 与 compare。
- 调用 tool bridge 形成 plan-only 或 scoped tool execution proposal。
- 请求 approval。
- 解释 failure 与 diff。

Agent 不直接修改 target branch，不绕过权限，也不持有 hidden official privilege。

### 5. AI 输出必须可追踪、可解释、可删除、可披露

AI-native 游戏不是“让模型随便吐内容”。平台必须让创作者与玩家知道：

- 生成物来自哪个 package、provider、inference、prompt-like input 和 source refs。
- 生成物是否 live-generated 或 pre-generated。
- 生成物是否经过 guardrail / policy / redaction。
- 生成物带来什么成本、延迟、失败风险。
- 生成物如何从 asset graph 中删除、替换或回滚。

## 推荐路线

### Experience Beta 0 — Thin Experience Runtime Contract

目标：定义普通 package-owned experience 如何连续运行、暂停、恢复、checkpoint、fork，并被 Agentic Forge 修改。

交付方向：

- Experience package authoring pattern。
- Session-state projection convention。
- Checkpoint asset convention。
- Failure/recovery event shape。
- Play surface state subscription pattern。
- Forge/Assist 与 experience session 的关联说明。

非目标：`kernel.v1.experience.*`、`kernel.v1.world.*`、`kernel.v1.turn.*`。

### Experience Beta 1 — First Real Playable Vertical Slice（已完成）

目标：尽早做一个可以连续玩 20–30 分钟的 AI-native experience，不等 State/Asset/Memory 全部补完。它不能是聊天壳、Tavern clone 或只有 prompt/response 的 demo。

已交付：

- `official/playable-creation-board` — package-owned playable creation board，包含 board/module/constraint/marker state。
- 能力：describe_contract / launch / project_state / render_payload / record_player_action / request_change / create_checkpoint / inspect_checkpoint / draft_recovery / bind_agent_run / explain_provenance。
- surface：experience_entry / play_renderer / forge_panel / assistant_action。
- record_player_action 产生 state_delta_asset_ref / projection_ref / sequence / provenance。
- request_change 输出 structured agent objective / allowed_change_kinds / risk / budget / bindable refs（不是聊天消息）。
- bind_agent_run 输出与 agentic-forge 的 scoped binding。
- explain_provenance 输出 player_action_event→state_delta_asset→checkpoint→agent_run→candidate→proposal→projection_rebuild 因果链。
- checkpoint / recovery 对齐 experience-runtime-lab 形状。
- Raw-secret blocking。
- 第三方 agentic-forge 替换 composition 证明无 official priority。
- CLI demo `ygg playable-board-demo`。
- Forge profile 自动加载。
- conformance 用例。

非目标：`kernel.v1.experience.*`、`kernel.v1.world.*`、聊天壳、assistant messages/conversation/prompt transcript。

这一步是 Yggdrasil 的真正产品证明。后续 state、asset、memory、observability 的最小实现范围由此 vertical slice 暴露的真实需求牵引。

### Experience Beta 2 — State + Asset Pipeline Alpha

目标：让体验状态和生成资产真正可追踪、可比较、可恢复。此阶段只交付 First Real Playable Vertical Slice 暴露出的最小必要集，其余内容进入后续 hardening。

交付方向：

- Content-addressed asset blobs。
- Asset provenance graph。
- Derived asset refs。
- AI-generated / live-generated metadata。
- Rights/licensing/disclosure metadata slots。
- State snapshot asset。
- State diff preview。
- Branch-aware asset/state views。
- Safe preview descriptors。
- Large output handling。
- Package-scoped asset permission checks。

非目标：完整媒体编辑器、统一 media schema、内核世界状态模型。

### Experience Beta 3 — Experience Observability

目标：让用户和创作者知道发生了什么、为什么失败、成本/延迟在哪里。此项应从 Experience Beta 1 起就作为验收条件出现，然后在本阶段系统化。

交付方向：

- Session health。
- Package health。
- Agent run health。
- Model/inference cost and latency summary。
- Proposal causal chain。
- Asset provenance graph view。
- Failure breadcrumbs。
- Stuck run detection。
- Guardrail/audit summary。

非目标：完整 APM、SaaS monitoring backend。

### Experience Beta 4 — Memory / Knowledge Package Alpha（已完成）

目标：普通包形式的长期记忆与知识，不进入内核。

已交付：

- `official/memory-lab`：能力包括 describe_memory_contract / record_memory / retrieve_memory / trace_retrieval / draft_memory_update / apply_memory_correction / draft_forget_redaction / branch_memory_view / explain_memory_provenance；surface 包括 forge_panel / assistant_action / home_card。
- Memory record 含 content_address / branch_ref / disclosure / knowledge_refs / source_refs。
- Branch-aware memory view（current_branch / all_branches / specified_branch / branch_diff）。
- Retrieval trace（关键词匹配，无 embedding/network）。
- Proposal-gated memory update（draft_memory_update 只产 proposal draft，不直接改持久状态）。
- User correction（apply_memory_correction 产出 correction shape，proposal-gated）。
- Forgetting / redaction workflow（draft_forget_redaction 产出 redaction plan，不直接删除）。
- Memory provenance（explain_memory_provenance 产出链，每步含 content_address）。
- Knowledge source refs（record_memory 和 request_change 支持 knowledge_refs）。
- `official/playable-creation-board` 新增 `memory_refs` / `knowledge_refs` / `retrieve_context_plan` 可选交叉引用（board 不依赖 memory-lab 才能运行）。
- 第三方替换证明：`thirdparty/memory-lab` + `examples/compositions/memory-lab-replacement/` composition。
- conformance 用例。
- 指南：`docs/guides/MEMORY_PACKAGE_AUTHORING.md`。

非目标：`kernel.v1.memory.*`、官方唯一 RAG、聊天记忆系统。

### Experience Beta 5 — Creator Loop Beta

目标：一个新创作者不读源码，只靠 docs、template、Forge，一天内做出可玩的 package。

交付方向：

- Better experience templates。
- Fixture runner UX。
- Reload flow polish。
- Composition diagnostics。
- Authoring walkthrough based on a real package。
- Package error explainability。
- Forge authoring workflow。

非目标：marketplace、creator monetization。

### Experience Beta 6 — Sharing / Distribution Alpha

目标：先支持可分享、可复现、可导入，再考虑市场。

交付方向：

- Export/import composition。
- Export/import branch/session bundle。
- Package-set lockfile。
- Compatibility/migration report。
- AI disclosure metadata bundle。
- Read-only shared session。
- Async fork sharing。

非目标：marketplace、package signing network、dependency resolver economy、hosted billing。

## 第一个真实体验的选择标准

第一个 vertical slice 的题材可以很小，但必须满足这些要求：

- 不是聊天 UI。
- 不是 Tavern clone。
- 不依赖内核题材语义。
- 有持续 package-owned state。
- 有生成 asset 或 state mutation。
- 有 Agentic Forge 介入的创作/修改行为。
- 有 branch/fork/compare。
- 有失败恢复与 provenance。
- 官方 reference experience 不得成为 canonical runtime。
- 至少一个关键能力可被第三方包替换。

更适合的形态包括：

- living sandbox fragment。
- procedural artifact playground。
- AI-directed scene/workshop。
- branching worldlet。
- playable creation board。

题材不是重点。重点是它能压测 state、asset、memory、proposal、branch、agent、inference、Forge。

## 成功指标

从这条路线开始，conformance 数量不再是核心成果，只是安全网。

核心指标应切换为：

- 一个玩家能否连续玩 20–30 分钟？
- 一个创作者能否一天内做出可玩的 package？
- 用户能否理解 agent 为什么提出某个改动？
- 用户能否 reject 改动且 session 不被污染？
- 用户能否 fork 并比较分支差异？
- 失败时用户能否理解并恢复？
- 生成 asset 是否可追踪、可预览、可披露、可删除？
- 第三方包是否能替换官方关键能力且保持体验可用？

## 红线

- 不把内容语义放进 kernel。
- 不把 cloud API adapter 升级为平台模型抽象。
- 不把 Agentic Forge 做成聊天产品或 coding-agent clone。
- 不让 official package 获得 hidden privilege。
- 不让 UI 读 runtime internals 或 SQLite。
- 不把 marketplace / billing / SaaS key hosting 提前变成主线。
- 不用 conformance 数字替代真实体验验证。

## 一句话方向

```text
Yggdrasil 已经证明“平台可以承载自由创作”；
下一步要证明“承载出来的东西值得玩、值得改、值得 fork、值得停留”。
```
