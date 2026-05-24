# Alpha 状态

> [English](./ALPHA_STATUS.en.md) · [中文](./ALPHA_STATUS.md)

这是 Yggdrasil 当前状态的快照。每完成一个里程碑就更新一次。下面每条都有代码和 conformance 用例支撑，明确标注 partial 或 deferred 的除外。

愿景与原则见 [`CHARTER.md`](CHARTER.md)、[`architecture/VISION.md`](architecture/VISION.md)、[`product/PLAY_CREATION_MODEL.md`](product/PLAY_CREATION_MODEL.md)。下一步见 [`roadmap/NEXT_STEPS.md`](roadmap/NEXT_STEPS.md)。

## 概要

- **conformance：** 427 个具名 CLI 用例通过，外加 crate 与 service 单元测试；115 个 v1 schema（63 methods + 45 events + 7 top-level）验证通过。
- **章程纪律：** 内核对内容无意见；官方包无特权；只走公开协议；入口形式平等；能力句柄、bindings 注入、Path A / Path B、conformance kit 与生成 SDK 已落地；可信路径阻断 raw secret，全部走 manifest 声明的 `secret_ref`；权限授权可重新水化；网络权限带审计与脱敏；通用流式与取消生命周期；外发执行有边界，默认全拒；公开 HTTPS git fetch 也走同样的 host policy / 审计 / 脱敏边界；出站一元、SSE/NDJSON/raw 流和 WebSocket 三个原语都有完成审计事件。
- **代码健康：** CLI、运行时各域行为、协议分发、in-process 处理器、事件存储——都已按域拆分，不再继续往单文件里堆。

平台基础已就位，下一阶段由真实 AI 原生可玩体验来牵引剩下的工作。

## 内核

- 不带内容的会话、只追加的不透明事件、清单驱动的能力包、能力机制、钩子机制、surface 贡献、提案生命周期、资产/分支/projection 底座。
- SQLite 事件日志，每会话单调递增的序号，可重新水化的底座。
- 用 JSON Schema 子集校验能力 I/O 和能力包声明的事件 payload。
- 身份：`host_admin`、`host_dev`、`package`、`human`、`assistant`、`anonymous`。human 与 assistant 身份支持作用域授权。
- 审计事件：`kernel/v1/permission.granted|revoked|denied`、`kernel/v1/package.*`（生命周期）、`kernel/v1/proposal.*`（生命周期）。
- 持久授权：grant/revoke 事件可在 SQLite-backed runtime 中重新水化。
- Contract V1 是公开平台规范：63 个协议方法、45 个事件类型、115 个 JSON Schema。`kernel.v1.cap.*`、`kernel.v1.audit.package`、能力句柄、bindings 注入、Path B、conformance kit 与 SDK 生成均为 implemented。

## 安全执行

- **`secret_ref` 引用：** 支持 `secret_ref:<vault>:<key>`、`secretRef:`、`secret-ref:`、`host:` 各种前缀。能力包通过引用提及 secret，原始值不出现在事件、提案、日志、审计里。
- **环境变量解析器：** host 拥有的解析器，带显式 allowlist。默认全拒；env 名要先放行才能解析。错误只带 env 名，绝不带原始值。
- **本地加密 secret store：** `secret_ref:store:NAME` 通过 `StoreSecretResolver` 从 `~/.yggdrasil/secrets.dat` 解析；`secret_ref:project:NAME` 先读项目级 store，再按 `secret_policy` 回退平台 store；store 使用 age(X25519) 加密，主密钥来自 OS keyring（延后启用）或 0600 本地 key 文件。
- **Raw secret 阻断：** 提案的 operations / expected effects 与资产 metadata 会被保守扫描，明显的 API key、token、password 字段被拒绝。资产内容和普通文本不扫描，避免误伤用户内容。
- **网络权限声明：** 清单中的 `permissions.network` 同时支持扁平 `hosts`（向后兼容）和结构化 `declarations`（带 `host`/`methods`/`purpose`）。无声明的能力包不能出网。官方包没有绕过。
- **外发审计与脱敏：** 每条出站请求都生成审计记录，只含身份、能力包 id、目标主机、方法、用途、脱敏状态、用到的 `secret_ref`，不含原始 body / header / 提示词 / 响应。
- **外发执行边界：** 内容无关的 HTTP 与 WebSocket executor trait。默认 deny-all（fail-closed），可切换为 fake executor（带 fixture，用于 conformance）或 live executor（HTTP 使用 reqwest + rustls，WebSocket 使用 tokio-tungstenite + rustls；默认关闭；HTTP 为 HTTPS-only，WebSocket 为 WSS-only；重定向 fail-closed；secret header 只在执行时注入，不进审计）。真实 live model / WebSocket outbound 必须通过 profile 与环境变量显式 opt-in；默认 conformance 不联网，真实 WebSocket smoke 还要求 `YGG_LIVE_WEBSOCKET_TESTS=1`。
- **协议方法：** `kernel.v1.outbound.audit` 列出某个能力包的出站审计事件；`kernel.v1.outbound.execute` 让普通能力包通过 host executor 发起一元出站请求；`kernel.v1.outbound.stream` 提供 SSE/NDJSON/raw 流式出站；`kernel.v1.outbound.websocket.open|send|close` 提供双向 WebSocket 出站。
- **完成审计事件：** `kernel/v1/outbound.execute.completed`、`kernel/v1/outbound.stream.completed`、`kernel/v1/outbound.websocket.completed` 覆盖三种出站原语；事件只记录状态、计数、耗时、executor kind、network_performed、redaction state 和 `secret_ref` 引用。
- **流式生命周期：** 流注册表跟踪进行中的流式调用，按序发出 `kernel/v1/stream.started|chunk|progress|ended|error|cancelled|timeout`。取消和超时阻断后续 chunk。非流式能力被拒绝。

## 公开协议与传输

- 规范的请求/响应信封，自带 host 绑定的身份上下文。调用方不能自己声称是某个能力包或 admin。
- 同一份 dispatcher 同时承载 HTTP `POST /rpc` 和 host JSON-RPC stdio（`ygg host-stdio`）。
- 通过 SSE 订阅事件，支持 `after_sequence` 回放和实时追尾。
- 基于 profile 的 `ygg host serve` 自动加载能力包，对外暴露 `/rpc` 与 SSE。
- WebSocket 与 TCP 传输留作后续工作；WASM 与远程入口在清单中已是一等形式，执行延后。

## 包执行

- `rust_inproc` 包通过 host 提供的 trait 和 catalog 执行。声明了 in-process provider 但 catalog 里找不到的清单会被拒绝。
- `subprocess` 包通过 stdio 上的 JSON-RPC 执行，支持握手、调用、超时、degraded 状态、重启、卸载即杀、stderr 日志捕获。
- `wasm` 与 `remote` 入口：清单已支持，执行延后。
- 路径 A（`entry.contract: "v1"`）接收能力句柄 bindings 并接受权限强制；路径 B（`entry.contract: "none"`）自包含运行，不接收 v1 权威，但生命周期仍可观察。
- 能力路由支持显式 provider 选择，以及精确匹配和 `^x.y` 简单版本约束。歧义时拒绝，除非调用方指定 `provider_package_id`。
- 钩子机制：确定性排序、能力包持有的处理器、payload 元数据修改、否决、卸载清理；覆盖 `kernel/v1/event.before_append|after_append` 与 `kernel/v1/capability.before_invoke|after_invoke`。

## 底座

- 资产注册表：不透明的 `id`/`mime`/`hash`/`size`/`origin_package_id`/`metadata`，可从 SQLite 重新水化。权限执行与内容寻址 blob 存储留待后续。
- 会话 fork / 分支沿革，可从事件日志重新水化。
- 通用 projection 注册表：通过 `kind_prefix` 与 `writer_package_id` 过滤事件来重建，写入 `kernel/v1/projection.updated`。包持有的 projection 执行留待后续。
- 项目运行时：`ProjectDescriptor`、`ProjectRegistry`、`~/.yggdrasil/projects/<id>/` 布局、项目级 secret policy、Home 项目卡、项目级 storage summary、redacted package failure summary，以及 `yg project list/info/status/start/stop` 已落地。
- Surface 贡献：带版本、slot、激活方式、所需权限、审批策略、metadata 的描述符。Slot 包括 `experience_entry`、`home_card`、`play_renderer`、`forge_panel`、`asset_editor`、`assistant_action`。通过 `kernel.v1.surface.contribution.list` 与 `.describe` 发现。
- 提案生命周期：`kernel.v1.proposal.create|get|list|approve|reject|apply`。当前只 apply 通用操作 `asset.put` 和 `projection.rebuild`。更广泛的事务和回滚留待后续。

## 官方能力包

全部是普通能力包，没有内核特权。位于 `packages/official/`，通过普通清单加载：

**平台基础**

- `package-lab`、`schema-tools`、`event-tools`、`composition-lab`、`asset-lab`、`projection-lab`、`assistant-lab`。
- 包安装基础已由 `official/git-tools-lab`、`official/integrity-lab` 与 `official/install-lab` 作为普通能力包落地；CLI 提供 `yg install` / `uninstall` / `list-installed` / `update` / `lockfile`。

**创作能力族**

- `persona-lab`、`knowledge-lab`、`context-lab`、`text-transform-lab`。

**模型接入**

- `model-connector-lab` —— 不出网的 provider 元数据、profile 校验、secret 脱敏、发现计划、兼容性报告。
- `model-provider-lab` —— 云 API adapter 实验室。覆盖 OpenAI / Anthropic / Gemini / OpenAI-compatible / OpenRouter / DeepSeek / xAI / Fireworks，提供请求构造、伪造调用、流式归一、live loopback 形状与各家 quirk。它不是平台模型抽象，也不是 API 网关。
- `model-routing-lab` —— 不做推理的 consumer-slot 绑定、路由计划、回退计划、参数归一。

**Agent 与推理**

- `pi-agent-runtime-lab` —— 参考 agent 包，no-network 的运行计划、trace 摘要、提案草稿、echo。
- `capability-tool-bridge-lab` —— 发现能力、预览权限、显式 provider 选择、调用/流式计划。Phase D 加入 `explain_tool_call`、`record_tool_observation`、`summarize_tool_risk`、`replay_tool_plan`、`plan_toolchain`，覆盖嵌套委派、target branch 写入、提示词注入、secret 外泄、出站扩张、大输出 redaction 等风险。
- `agentic-forge-lab` —— Agentic Forge Beta 的核心包：能力包持有的运行生命周期、工作状态、计划图、scratch branch / candidate / compare / promote、推理节点（确定性 / 录制 / 云适配计划 / 本地 fake）、replay、输出校验、9 类失败 taxonomy。
- `inference-local-lab` —— 不依赖云 API、HTTP、bearer token 的本地 fake 推理 provider，证明推理接缝可以脱离这些。
- `inference-playtest-lab` —— Ygg-native 的「推理 → 提案 → 审视 → 批/拒 → 应用 → fork」纵切片。

**体验**

- `experience-runtime-lab` —— 体验运行时契约：体验描述符、状态投影、checkpoint、recovery、Play/Forge/Assist surface 绑定。
- `playable-creation-board` —— 第一个真实可玩的纵切片。包持有 board / module / constraint / marker 状态，14 个能力，4 个 surface。
- `experience-observability-lab` —— 包持有的可观测性：会话健康、能力包健康、agent 运行健康、提案因果链、cost/latency 摘要、失败面包屑、guardrail 摘要。
- `memory-lab` —— 长期记忆与知识：记录、检索、检索追踪、提案审批门控的更新、修正、forget/redaction、按分支视图、provenance。
- `sharing-lab` —— 分享与分发：composition bundle 导入导出、分支/会话 bundle 清单、包集 lockfile、兼容性报告、AI 披露元数据、只读分享清单、异步 fork 计划。不带市场、计费、签名网络。
- `playable-seed`、`blank-experience` —— 参考与最小体验。

**存储与外部项目**

- `storage-lab` —— 存储/数据契约预览：分层模型、backend class 候选、包级状态库、文档 CRUD 预览、blob 内容寻址契约证明、projection 物化、检索 / 向量 / 多模态 provider 契约。
- `tdb-retrieval-lab` —— TDB 作为检索 / 多模态 provider 的契约；不是事件日志权威。
- `project-intake-lab` —— 外部项目分类、栈检测、npm 生命周期风险、工作区计划、adapter 计划、wrapper / fixture / readiness 预览。不出网、不动文件系统。
- `workspace-lab` —— 工作区行动策略边界，10 项行动 taxonomy，deny-by-default 假执行器，确定性 fixture 工作区。

**第三方替换证明**

- `thirdparty/playable-seed`、`thirdparty/agent-runtime`、`thirdparty/agentic-forge`、`thirdparty/memory-lab` —— 证明对应官方包都可被第三方替换，没有官方优先级。

Forge profile（`profiles/forge-alpha.yaml`）会自动加载这些包以及示例 fixture 包。

## TypeScript SDK

`sdk/typescript/` 下：

- `subprocess` —— 子进程能力包脚手架与模板运行时。
- `secure-execution` —— `secret_ref` 构造与校验、网络声明、出站审计、伪造流帧客户端。
- `inference-capability` —— transport-neutral 推理契约。
- `model-provider-adapter` —— 云 provider adapter helper。
- `ygg-agent-adapter` —— 把 Ygg 能力映射为 pi 风格 tool。
- `agentic-forge` —— 运行生命周期、计划图、工作状态、candidate / compare / promote、推理节点、tool bridge v2 helper。
- `experience-runtime` —— 体验运行时类型与构造器。
- `text-surface` —— 前端文字 surface helper（流式 buffer、frame 适配、滚动锚、字体加载）。

`text-surface`、`agentic-forge`、`inference-capability` 等都自带纯 TS 自测。

## Contract v1 与 SDK 生成

- `docs/spec/KERNEL_V1_CONTRACT.md` 是公开平台规范。
- `docs/spec/v1/schemas/` 是 SDK 和 conformance 的单一可信源：63 methods、45 events、7 top-level，共 115 个 schema。
- `sdk/typescript/kernel-sdk/` 与 `sdk/rust/yg-kernel-sdk/` 由 schema 生成；TypeScript 包可通过 npm、工作空间路径或自行 codegen 使用。
- `yg conformance package --contract v1 --path <package>` 提供第三方包 8 项验收检查。

## 包模板

`ygg init-package --template <name>`：`basic`、`experience`、`play-renderer`、`forge-panel`、`assistant-action`、`asset-editor`、`full-surface`、`networked`、`streaming`、`agent-runtime`、`experience-runtime`、`playable-board`、`playable-experience`。生成的包默认安全：no raw secret、不隐式联网。

## 包安装（新，Round 10A）

| 能力 | 状态 |
|---|---|
| manifest.requires 字段 | implemented |
| Lockfile schema (`yggdrasil.lock.v1`) | implemented |
| official/git-tools-lab (gix) | implemented |
| official/integrity-lab (sequoia GPG + sha256) | implemented |
| official/install-lab orchestrator | implemented |
| yg install / uninstall / list-installed / update / lockfile CLI | implemented |
| ~/.yggdrasil 文件系统约定 | implemented |
| 交互式同意提示 | implemented |
| 静态 conformance 集成 | implemented |
| GPG 签名验证 | implemented |
| 循环依赖检测 | implemented |
| 真实 GitHub smoke (opt-in) | implemented |
| Sigstore 验签 | deferred |
| Tauri UI 安装 | deferred |
| 自动更新守护 | deferred |
| 二进制包分发 | deferred |
| yg gc 孤立 store 回收 | deferred |

Round 10A.1 后安装默认值已放宽：HTTPS-only、内容哈希、原子写入始终启用；签名验证与 conformance 阻断分别通过 `--require-signed` / `--strict` opt-in。

## Round 10A.1 — Install Simplification + Secret Store

| 能力 | 状态 |
|---|---|
| Install defaults relaxed (cargo/npm/pip baseline) | implemented |
| --require-signed / --strict opt-in flags | implemented |
| Single-line consent prompt | implemented |
| official/secret-store-lab 加密存储 | implemented |
| StoreSecretResolver | implemented |
| CompositeSecretResolver (env + store) | implemented |
| age 加密 (X25519) + 0600 文件权限 | implemented |
| OS keyring 集成 | deferred (libdbus-sys system dep) |
| YdlTavern API Connections wired | implemented |
| `yg secret put / list / delete` CLI | deferred |


## Round 10A.2 — Steam-Game Project Concept

| 能力 | 状态 |
|---|---|
| P0 Wave 1: secret resolver host-profile wiring | implemented |
| Project as first-class runtime concept | implemented |
| ProjectDescriptor + ProjectId + ProjectType + SecretPolicy | implemented |
| ~/.yggdrasil/projects/<id>/ filesystem layout | implemented |
| secret_ref:project:NAME with platform fallback | implemented |
| ProjectRegistry (in-memory + disk scan) | implemented |
| ProtocolContext.session_id propagation | implemented |
| Install detection (native vs external) | implemented |
| External project wizard (wrap / workspace) | implemented |
| yg project list/info/status/start/stop | implemented |
| yg uninstall with archival prompt | implemented |
| kernel.v1.project.list/get/start/stop/status | implemented |
| kernel/v1/project.installed/started/stopped/uninstalled | implemented |
| Home surface project cards | implemented |
| YdlTavern project.yaml | implemented |
| Multi-tenant project_id in ProtocolContext | deferred (Round 11+) |
| Project archive auto-cleanup beyond 30 days | deferred |

## Round 10A.3 — End-to-End Real Path

| 能力 | 状态 |
|---|---|
| huggingface-fetcher tests passing | implemented (Wave 1) |
| Surface bundle resolution metadata-driven | implemented (Wave 2) |
| kernel.v1.surface.resolve_bundle | implemented |
| host /surface-bundles/<prefix>/<file> route | implemented |
| /surface-bundles/projects/<id>/<file> route | implemented |
| project.start opens project session + sets metadata.project_id | implemented (Wave 3A) |
| project.start returns session_id + already_running | implemented |
| project.get/status return running_session_id | implemented |
| project.stop emits + closes project session | implemented |
| Surface receives session_id via initialProps | implemented (Wave 3B) |
| TavernProvider.sendMessage invokes engine model.live_call | implemented |
| API Connections drawer scope toggle (platform/project) | implemented |
| Engine manifest declares secret_ref:project:* | implemented |
| Streaming response UX in surface | implemented (Round 10A.4) |
| Multi-tenant project_id in ProtocolContext | deferred (Round 11+) |

## Round 10A.4 — Streaming UX

| 能力 | 状态 |
|---|---|
| Surface-host stream postMessage protocol | implemented |
| stream.subscribe / stream.frame / stream.ended / stream.error / stream.unsubscribe | implemented |
| Host bridge taps client.subscribeEvents, filters by stream_id | implemented |
| streamCapability helper in YdlTavern host-rpc | implemented |
| AsyncIterable<StreamFrame> consumption + iterator early-return cleanup | implemented |
| TavernProvider.sendMessage streaming branch | implemented |
| Progressive assistant message updates (chunk delta accumulation) | implemented |
| Engine `final` frame defensive handling | implemented |
| cancelGeneration action | implemented |
| Stop button swap in SendForm when isGenerating | implemented |
| Multi-concurrent generation in single chat | deferred |
| Stream rate / token-rate indicator UI | deferred |
| Realtime/WebSocket streaming UX | deferred (separate capability path) |

Round 10A.4 不改变 conformance 与 schema 数量：仍为 427 个 conformance 用例、115 个 v1 schema。YdlTavern 测试摘要：surface 110/4（Round 10A.3 为 94/4，Wave 1B + Wave 2 streaming 测试新增 16 个）、engine 90/2、engine-core 307/0、golden harness 20/20 perfect。

## Completed（S-track shell / release）

- **Web client（S1）：** `clients/web` 已切到 Vite dev/build，并升级为 React 19 + Tailwind v4 + Motion + Radix + Phosphor SPA。Home/Settings/Install/Project frame/Toast 全部以公开协议为唯一数据源（HTTP `/rpc` + SSE + 可选 postMessage stream bridge）；iframe-based SurfaceHost 可挂载第三方 surface bundle，并通过显式 `postMessage` RPC bridge 与宿主通信。详见 [`design/PLATFORM_UI_DESIGN.md`](design/PLATFORM_UI_DESIGN.md)、[`../clients/web/README.md`](../clients/web/README.md)、[`guides/SURFACE_HOSTING.md`](guides/SURFACE_HOSTING.md)。
- **Desktop wrapper（S2）：** `clients/desktop` 提供 Tauri 2.x wrapper，生产模式嵌入 `clients/web/dist`，开发模式指向 Vite dev server。v0 不自动启动 `ygg-cli host serve`；用户仍需单独运行 host。
- **Release pipeline（S3）：** GitHub Actions CI 与 `v*` tag release workflow 已落地，构建跨平台 Tauri 安装包并创建 draft release。`scripts/release-version.sh` 同步 Cargo、Web package、desktop package 与 Tauri 配置。构建说明见 [`../BUILDING.md`](../BUILDING.md)，变更记录见 [`../CHANGELOG.md`](../CHANGELOG.md)。签名、公证、自动更新未启用。

## Web shell（`clients/web`）

平台用户面 chrome——Home、Settings、Install 流程、Project frame、Toast 系统。基于 React 19 + Tailwind v4 + Motion + Radix + Phosphor 的 SPA，由 Vite 构建。视觉规则与设计系统见 [`design/PLATFORM_UI_DESIGN.md`](design/PLATFORM_UI_DESIGN.md)；shell 详细文档见 [`../clients/web/README.md`](../clients/web/README.md)。

- **Home：** 项目货架（卡片网格 + 状态 pill + Hero + utility strip + 活动 timeline + 工坊工具 bento），数据来自 `kernel.v1.project.list`，磁盘用量来自项目 `storage_summary`。`⌘N` 打开 Install 模态。
- **Settings：** 五个面板都接真实数据。
  - API Connections — `official/secret-store-lab/{list,put,delete}_secret` + health。UI 永远不读 raw secret 值。
  - Installed Packages — `kernel.v1.package.list` + 项目标记 + Cmd/Ctrl+F focus。
  - Profiles — `kernel.v1.host.diagnostics`（active profile、packages_loaded、network allowlist）。
  - Storage — storage area summary + 真实 event store kind（sqlite/postgres/memory），不在 Web UI 暴露 host 绝对路径。
  - About — 平台身份、license、links、致谢。
- **Install 流程：** 三步 modal 通过 `kernel.v1.capability.invoke` 调用 `official/install-lab` 的 `resolve_plan` / `detect_kind` / `execute_plan`；原生项目走快速通道，外部项目进入 wrap-vs-workspace wizard。没有 `kernel.v1.install.*`。
- **Project Frame：** 60px 平台 topbar + 40px 项目 topbar（项目名/状态 pill/Stop/Audit log）+ iframe SurfaceHost 挂载项目自有前端。
- **Failure Modal：** Deep Rust accent stripe、诊断/影响双列、redacted stderr 日志面板（含 Copy log）、Restart/Stop-and-uninstall/Close 三选项；数据来自 `kernel.v1.package.list/status/logs`，不复制 raw log。
- **Toast 系统：** 5 个 variant（info/success/warning/error/progress），右下队列，prefers-reduced-motion 自动收敛。
- **响应式与暗色模式：** 显式 `data-theme` 切换（system/light/dark）；`@custom-variant dark` 把 Tailwind `dark:` 绑定到属性；modal overlay 用单独的 `--color-overlay` token 不随主题翻转；`prefers-reduced-motion` 收敛动效；`:focus-visible` 键盘导航 ring。
- **SurfaceHost：** 通过 sandboxed iframe 挂载第三方 Web surface bundle；默认没有 kernel access，只有宿主显式配置的 bridge 能调用公开协议。流式订阅通过 postMessage 桥接 `kernel/v1/stream.*`。
- **没有官方包硬编码——shell 和别的客户端一样是公开协议的客户端。**

## 创作流程

- `ygg init-package` 生成 Python 或 TypeScript 子进程包脚手架。`--template` 控制 surface 描述符；`--language *-experience` 在不指定模板时仍生成旧版 4-surface 体验，兼容旧行为。
- `ygg init-composition` + `ygg composition check` 提供本地 composition 流程，支持 v2 字段（标题、描述、可选包、所需能力、默认激活、权限期望、替换候选、兼容性说明）。
- `ygg package check` 输出结构化诊断：入口类型、信任级别、能力数、按 slot 分组的 surface、权限摘要、沙箱策略；对无能力或无 surface 的包给出警告。
- `ygg package conformance` 在本地验证生成的包。
- `ygg package reload <manifest>` 把包加载进内存运行时、重启（仅子进程）、输出前后状态和日志数、再卸载。
- `ygg package run-fixture` 用确定性 fixture 输入调用所有非流式能力，输出 JSON 摘要。
- `ygg play-create-demo` 端到端跑通空白游创循环。
- `ygg perf baseline` 跑确定性性能基线（in-process 调用、官方能力调用、事件存储 append/list/range、composition check、profile 加载、子进程 echo），输出文本或 JSON。详见 [`performance/BASELINE.md`](performance/BASELINE.md)。

## 代码组织

- `crates/ygg-cli/src/main.rs` 是薄入口。CLI 类型在 `cli.rs`，命令在 `commands/`，包模板在 `templates/`，conformance 用例按域分模块在 `conformance/`，使用结构化的 `ConformanceCase { id, tags, run }` 注册表，支持 `--list`、`--case`、`--tag`、`--fail-fast`、`--slowest`，附带 per-case 用时和最慢 N 报告。
- `crates/ygg-runtime/src/runtime/` 按 session、events、packages、capabilities、hooks、permissions、assets、branches、projections、proposals、protocol dispatch 分模块；`runtime/mod.rs` 保持公开 `Runtime<S>` API。
- 协议方法的元数据与分发共享 `KernelMethod` 这一份事实来源，并有注册表 / 分发的一致性单测。
- `crates/ygg-runtime/src/inproc/` 把官方包行为按域拆开，公共 helper 走 provider package + 本地能力名路由，不再用 suffix-only 兜底。

这次拆分不改变行为，只是让后续新增能力包、conformance、handler 时仍然可审查。

## Conformance

`cargo run -p ygg-cli -- conformance` 跑 427 个具名 CLI 用例。支持：

- `--list` 列出 id 与 tag；
- `--case <pattern>` 子串过滤；
- `--tag <tag>` 按 tag 过滤；
- `--fail-fast` 首个失败即停；
- `--slowest <N>` 显示最慢 N 个。

每个用例有 tag（runtime / event / capability / package / subprocess / official / network / outbound / stream / agentic / experience / memory / sharing / secret / composition / replacement / surface / protocol / permission / hook / host / asset / projection / substrate / storage / live / external_project / project_intake / workspace_lab / retrieval 等）。详见 [`performance/CONFORMANCE_FEEDBACK.md`](performance/CONFORMANCE_FEEDBACK.md)。

外加 `cargo test --workspace` 下的 crate 与 service 单测，以及 `npm run check --prefix clients/web` / `npm run build --prefix clients/web` 检查 Web shell。

## Partial（已开始但未做完）

- 能力包身份的 `event.subscribe` 权限。
- 能力包持有的钩子处理器的超时 / 错误审计。
- 能力 provider 的持久选择策略（超出单次调用显式选择）。
- 更丰富的资源策略（filesystem 强制矩阵）。
- 内容寻址的资产 blob 存储与能力包身份的资产权限：稳定的 content-address helper 与元数据约定已完成；完整 blob 存储与运行时权限执行未完成。
- 包持有的 projection 执行。
- 更丰富的失败监控与健康检查。
- 更广的传输一致性覆盖。
- Desktop release code signing / notarization、auto-updater、真实应用图标、桌面 wrapper 管理 host 子进程。
- Surface lifecycle callback（如 `onClose`、`onProposalDraft`）与跨源 surface bundle allowlist。
- `kernel.v1.session.get|list`、`kernel.v1.package.describe`、`kernel.v1.capability.describe`、`kernel.v1.extension_point.describe`、`kernel.v1.host.principal`、`kernel.v1.host.ping` 完整暴露。

## Deferred（明确不在内核范围）

这些都将以普通能力包或后续工作出现，不属于内核：

- 对话运行时、提示词、模型、采样、消息 / 回合语义。
- 记忆模型、检索、摘要、agent 循环、导演。
- 世界、场景、角色、规则、骰子、背包语义。
- SillyTavern 兼容由独立项目 YdlTavern 承担，跑在 Yggdrasil 之上（见 [`tavern/TAVERN_COMPAT.md`](tavern/TAVERN_COMPAT.md)）。
- 生产级长期自治 agent、多 agent 协作、生产级记忆系统、更完整的 live-ops。
- 外部游戏引擎桥接（UE5、Godot、Unity、Web 客户端）。
- 市场、包签名、依赖解析（本地分享 proof 已完成，见 [`guides/SHARING_DISTRIBUTION.md`](guides/SHARING_DISTRIBUTION.md)）。
- 最终 UI 视觉设计、完整 Studio、ComfyUI 风格节点编辑器。
- WASM 与远程包执行。

## 如何核对这份快照

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- conformance --list
cargo run -p ygg-cli -- conformance --tag sharing --slowest 3
cargo run -p ygg-cli -- play-create-demo
npm run check --prefix clients/web
npm run build --prefix clients/web
```

任何一步失败时，以代码为准，更新这份文档。

## 延伸阅读

- [`CHARTER.md`](CHARTER.md) —— 不变的根本原则。
- [`architecture/`](architecture/README.md) —— 架构、内核、能力包契约、扩展点、事件、生命周期。
- [`product/`](product/README.md) —— 游创立场与体验牵引平台路线。
- [`protocol/PROTOCOL_V0.md`](protocol/PROTOCOL_V0.md) —— 公开协议。
- [`spec/`](spec/README.md) —— 可执行契约矩阵与 conformance 路线图。
- [`guides/`](guides/README.md) —— 能力包创作指南。
- [`roadmap/NEXT_STEPS.md`](roadmap/NEXT_STEPS.md) —— 当前与下一阶段。
