# Yggdrasil

> [English](./README.en.md) · [中文](./README.md)

**面向 AI 原生世界、游戏、故事和游玩的扩展驱动创作平台。**

一个内核 + 一份契约：中心小、稳定、不带主观意图。在它之上，由能力包（capability package）组成的开放生态提供平台中每一个有意义的概念——角色、提示词、模型、agent、世界、规则、记忆，皆由包提供。

```text
┌──────────────────────────────────────────────┐
│  Web shell · CLI · 第三方客户端                  │   走公开协议
├──────────────────────────────────────────────┤
│  Public protocol  ·  /rpc + SSE              │
├──────────────────────────────────────────────┤
│  Capability packages（官方包 = 第三方包）         │   manifest 驱动
├──────────────────────────────────────────────┤
│  内核：sessions · events · permissions · ...   │   内容无关
└──────────────────────────────────────────────┘
```

## 为什么做这个

今天大多数 AI 原生创作工具，把使用者切成两半：消费成品体验的玩家，和构建体验的开发者。**Yggdrasil 拒绝这种切分。**

玩家可以审视当前 session、让 assistant 修改、fork 它、替换其中某个能力包，再把改动反馈出去。创作者面对同一份公开协议、同样的能力包、同样的 surface。底层底座在两个方向上完全相同。

完整的产品立场见 [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.md)。

## 重心所在

- 内核只承担承载能力包的职责，仅此而已。
- 所有有意义的概念都由能力包提供。
- 官方包没有任何特权。同一份 manifest、同一套能力网络、同一道权限闸门。
- 创作者可以自由组合、替换或自己写新的能力包。

平台的职责是让激进的 AI 原生创作成为可能，而不是给某条「官方路径」开特权。

## 当前状态

平台基础已就位，正进入由真实 AI 原生 playable experience 牵引的 Experience-Led Platform Beta 阶段。

- **320 个具名 conformance 用例** + crate / service 单元测试，全部通过。
- 已完成阶段：Platform Foundation Alpha、Play/Forge Surface Contract Beta、Secure Execution Substrate Alpha、Optional Text Engine Alpha、Agent Infrastructure Alpha、Model Provider Integration Alpha、Live Model Calls Alpha、Creative Inference Capability Alpha、Agentic Forge Beta、Experience-Led Platform Beta（Beta 0–6）、Performance & Code Health Beta、External Project Operating Plane Alpha、Storage Backend Neutrality Alpha、PostgreSQL + TDB Integration Alpha、Real TDB Rust Adapter Alpha。

可执行快照见 [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.md)；下一阶段见 [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.md)。

## 仓库一览

```text
crates/      Rust 内核与运行时
  ygg-core/      内核类型与契约（内容无关）
  ygg-runtime/   运行时主机：events / packages / capabilities / hooks /
                 surfaces / proposals / assets / branches / projections
  ygg-service/   公开协议层（HTTP /rpc，事件 SSE 订阅）
  ygg-cli/       host 模式、manifest 工具、能力包脚手架、conformance

clients/web/   走公开协议的 Home / Play、Forge、Assist Web shell

packages/official/   作为普通 manifest 加载的官方基础能力包
profiles/            host profile，批量自动加载能力包
examples/            示例 manifest 与 fixture 包

sdk/typescript/      subprocess 包脚手架与领域 SDK
docs/                架构、协议、规范、路线图、产品文档
integrations/        上游项目调研 ledger（pi、TavernHeadless、pretext、TDB...）
```

## 配套能力

**内核与执行**

- 内容无关 session、不透明事件、SQLite 持久日志、可重新水化底座
- 真正的 `rust_inproc` 与 subprocess 执行，hook fabric，能力 fabric
- Principal 与作用域权限，proposal/approval 生命周期

**安全执行**

- `secret_ref` 引用、`EnvSecretResolver` allowlist、host-owned 解析
- 网络权限声明，outbound audit/redaction，公开 `kernel.outbound.execute`
- `LiveHttpOutboundExecutor`（HTTPS-only、默认关闭、redirect fail-closed）
- 通用 streaming/cancel/timeout 生命周期

**官方能力包**（全部走普通 manifest，无内核特权）

- 平台基础：`composition-lab`、`asset-lab`、`projection-lab`
- 创作工具：`persona-lab`、`knowledge-lab`、`context-lab`、`text-transform-lab`
- 模型接入：`model-connector-lab`、`model-provider-lab`、`model-routing-lab`（OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks）
- Agent 基础设施：`pi-agent-runtime-lab`、`capability-tool-bridge-lab`、`agentic-forge-lab`
- 体验：`playable-creation-board`、`experience-runtime-lab`、`experience-observability-lab`、`memory-lab`、`sharing-lab`、`playable-seed`
- 推理：`inference-local-lab`、`inference-playtest-lab`
- 存储/外部项目：`storage-lab`、`tdb-retrieval-lab`、`project-intake-lab`、`workspace-lab`
- 基础实验：`package-lab`、`schema-tools`、`event-tools`、`assistant-lab`、`blank-experience`

**SDK（TypeScript）**

- `subprocess` 子进程包脚手架
- `secure-execution`、`agentic-forge`、`ygg-agent-adapter`
- `inference-capability`、`model-provider-adapter`、`experience-runtime`
- `text-surface`（前端文字界面 helper）

**Web shell**

- Home / Play、Forge、Assist 三个深表面，全部走公开协议
- 可选前端文字引擎（fallback + 可选 Pretext，动态导入）
- Forge 文字预览、agent / 体验 / 存储 / proposal 观测面板

## 快速上手

启动 host：

```bash
cargo run -p ygg-cli -- host serve \
  --http 127.0.0.1:8787 \
  --profile profiles/forge-alpha.yaml
```

类型检查 Web shell：

```bash
tsc -p clients/web/tsconfig.json --noEmit
```

跑完整 conformance 套件：

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

只用公开协议跑通空白游创循环：

```bash
cargo run -p ygg-cli -- play-create-demo
```

更多命令（manifest、package、composition、host 模式、第三方创作循环、模板）见 [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md)。

## 文档导航

每篇开发文档都同时提供英文与简体中文版本，文件顶部的双语导航 blockquote 可在两种语言间切换。

[`docs/`](docs/README.md) 索引按主题分组：架构、协议、规范、产品、能力包创作、性能、路线图。

按受众的最短读路径：

| 你想 | 先读 |
|---|---|
| 理解平台立场 | [`docs/CHARTER.md`](docs/CHARTER.md) → [`docs/architecture/VISION.md`](docs/architecture/VISION.md) → [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.md) |
| 理解架构 | [`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.md) → [`docs/architecture/PLATFORM_KERNEL.md`](docs/architecture/PLATFORM_KERNEL.md) → [`docs/architecture/CAPABILITY_PACKAGE.md`](docs/architecture/CAPABILITY_PACKAGE.md) |
| 接入公开协议 | [`docs/protocol/PROTOCOL_V0.md`](docs/protocol/PROTOCOL_V0.md) → [`docs/spec/KERNEL_V0_ALPHA_CONTRACT.md`](docs/spec/KERNEL_V0_ALPHA_CONTRACT.md) |
| 写第一个能力包 | [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md) |
| 写 agent / 模型 / 体验包 | [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](docs/guides/AGENT_PACKAGE_AUTHORING.md)、[`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](docs/guides/MODEL_PROVIDER_INTEGRATION.md)、[`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md`](docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md) |
| 看当前状态 | [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.md) |
| 看下一步 | [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.md) |

## 延后事项

下面这些方向有价值，但不属于内核——它们都将以普通能力包的形态到来：

- SillyTavern 兼容（[`docs/tavern/TAVERN_COMPAT.md`](docs/tavern/TAVERN_COMPAT.md)）
- 生产级长期自治 agent、多 agent 协作、生产记忆系统、世界模拟、director
- 外部游戏引擎接入（UE5、Godot、Unity、Web 端）
- 完整 Studio、ComfyUI 风格节点编辑器、市场
- 最终视觉设计

## 协议

Yggdrasil 以 GNU Affero General Public License v3.0（AGPLv3）发布，详见 [`LICENSE`](LICENSE)。
