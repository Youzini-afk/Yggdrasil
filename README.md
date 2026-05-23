# Yggdrasil

> [English](./README.en.md) · [中文](./README.md)

**一个面向 AI 原生世界、游戏、故事与游玩的可扩展创作平台。**

它由两部分构成：一个稳定、克制、不带主观意图的内核，和一个开放的能力包生态。平台中每一个有意义的概念——角色、提示词、模型、agent、世界、规则、记忆——都来自能力包，不是内核。

```text
┌──────────────────────────────────────────────┐
│  Web shell · CLI · 第三方客户端                 │   走公开协议
├──────────────────────────────────────────────┤
│  公开协议   ·   /rpc + SSE                    │
├──────────────────────────────────────────────┤
│  能力包（官方包 = 第三方包）                       │   清单驱动
├──────────────────────────────────────────────┤
│  内核：会话 · 事件 · 权限 · ...                 │   对内容无意见
└──────────────────────────────────────────────┘
```

## 为什么做这个

今天大多数 AI 原生创作工具把使用者切成两半：消费成品体验的玩家，和构建体验的开发者。**Yggdrasil 拒绝这种切分。**

玩家可以审视当前会话、让 assistant 修改、fork 它、替换其中某个能力包，再把改动反馈出去。创作者面对同一份公开协议、同样的能力包、同样的 surface。底座在两个方向上完全相同。

完整的产品立场见 [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.md)。

## 重心所在

- 内核只承载能力包，不干别的。
- 所有有意义的概念都由能力包提供。
- 官方包没有任何特权——同一份清单，同一套机制，同一道权限闸门。
- 创作者可以随意组合、替换、或自己写新的能力包。

平台的职责是让激进的 AI 原生创作成为可能，不是给某条「官方路径」开特权。

## 当前状态

平台底座已经搭好。下一阶段不再继续摊大表面积，而是用真实的可玩体验来牵引剩下的工作。

- 360 个具名 conformance 用例 + crate / service 单元测试，全部通过。
- 内核内容无关，官方包无特权，公开协议唯一入口。
- 安全执行、提案审批、流式生命周期、模型接入、agent 基础设施都已落地。
- Web shell 已切到 Vite dev/build；`clients/desktop/` 提供 Tauri 2.x 桌面 wrapper，`v*` tag 通过 GitHub Actions 构建跨平台安装包。
- perf baseline 现在记录 p50/p95/p99 + memory + env/git，支持 `--compare` + `--threshold-pct`，并已提交 `perf/baseline.json`。

详细情况见 [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.md)；下一步方向见 [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.md)。

## 仓库一览

```text
crates/                Rust 内核与运行时
  ygg-core/              内核类型与契约（对内容无意见）
  ygg-runtime/           运行时主机：会话、事件、能力包、能力、钩子、
                         surface、提案、资产、分支、projection
  ygg-service/           公开协议层（HTTP /rpc、事件 SSE 订阅）
  ygg-cli/               host 模式、清单工具、能力包脚手架、conformance

clients/web/           Vite + plain TS 的 Home / Play、Forge、Assist Web shell
clients/desktop/       Tauri 2.x 桌面 wrapper（嵌入 web shell）

packages/official/     通过普通清单加载的官方基础能力包
profiles/              host profile，批量自动加载能力包
examples/              示例清单与 fixture 包

sdk/typescript/        子进程能力包脚手架与领域 SDK
docs/                  架构、协议、规范、路线图、产品文档
integrations/          上游项目调研记录（pi、TavernHeadless、pretext、TDB...）
```

## 配套能力

**内核与执行**

- 对内容无意见的会话、不透明事件、SQLite 持久日志、可重新水化的底座。
- 真实的 in-process 与子进程包执行，钩子机制，能力机制。
- 身份模型与作用域权限，提案与审批生命周期。

**安全执行**

- `secret_ref` 引用、manifest `permissions.secret_refs` 声明、host 拥有的环境变量解析器与 allowlist。
- 网络权限声明，外发请求的审计与脱敏，公开协议出站三件套：一元 `kernel.outbound.execute`、SSE/NDJSON/raw `kernel.outbound.stream`、双向 `kernel.outbound.websocket.*`。
- 真实 live HTTP / WebSocket 出站执行器（默认关闭；需 opt-in profile + provider env；HTTP 为 HTTPS-only，WebSocket 为 WSS-only，重定向 fail-closed）。真实 WebSocket smoke 还需要设置 `YGG_LIVE_WEBSOCKET_TESTS=1`。
- 子进程 TypeScript SDK 的 `kernelClient` 可从 subprocess 包发起受权限约束的 reverse kernel calls，并支持 `kernelClient.openWebSocket`。
- 公开 HTTPS git 安装通道：`kernel.outbound.git_fetch`、profile 级 lockfile、`official/package-installer-lab`。
- 通用的流式与取消生命周期。

**官方能力包**（全部走普通清单，没有内核特权）

- 平台基础：composition / asset / projection。
- 创作工具：persona / knowledge / context / text-transform。
- 模型接入：model-connector / model-provider / model-routing（OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks）。
- Agent：pi-agent-runtime / capability-tool-bridge / agentic-forge。
- 体验：playable-creation-board、experience-runtime、experience-observability、memory、sharing、playable-seed。
- 推理：inference-local、inference-playtest。
- 存储与外部项目：storage、tdb-retrieval、project-intake、workspace。
- 基础实验：package / package-installer / schema-tools / event-tools / assistant / blank-experience。

**TypeScript SDK**

- `subprocess` —— 子进程能力包脚手架。
- `secure-execution`、`agentic-forge`、`ygg-agent-adapter`。
- `inference-capability`、`model-provider-adapter`、`experience-runtime`。
- `text-surface` —— 前端文字 surface helper。

**Web shell**

- Home / Play、Forge、Assist 三个一等 surface，全部走公开协议。
- plain TypeScript SPA，通过 Vite 提供 dev/build/preview；不引入 React 或前端框架。
- iframe SurfaceHost 可以挂载第三方 surface bundle，例如 `@ydltavern/surface`，并通过显式 postMessage bridge 与宿主通信。
- 可选前端文字引擎（自带 fallback，可选加载 Pretext）。
- Forge 文字预览，agent / 体验 / 存储 / 提案 观测面板。

**桌面与发布**

- `clients/desktop/` 是 Tauri 2.x wrapper，生产模式嵌入 `clients/web/dist`。
- v0 不内置启动 `ygg-cli host serve`；用户仍需单独运行 host。
- `v*` tag 触发 GitHub Actions release workflow，生成 Linux / macOS / Windows 安装包草稿；签名、公证、自动更新仍未启用。

## 快速上手

启动 host：

```bash
cargo run -p ygg-cli -- host serve \
  --http 127.0.0.1:8787 \
  --profile profiles/forge-alpha.yaml
```

构建或检查 Web shell：

```bash
npm run check --prefix clients/web
npm run build --prefix clients/web
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

更多命令（清单、能力包、git 安装、composition、host 模式、第三方创作循环、模板）见 [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md) 与 [`docs/guides/GIT_PACKAGE_INSTALLATION.md`](docs/guides/GIT_PACKAGE_INSTALLATION.md)。

## 文档导航

每篇开发文档都有英文与简体中文两版，文件顶部的双语 blockquote 可在两种语言间切换。[`docs/`](docs/README.md) 按主题分组：架构、协议、规范、产品、能力包创作、性能、路线图。

按目的的最短读路径：

| 你想 | 先读 |
|---|---|
| 理解平台立场 | [`docs/CHARTER.md`](docs/CHARTER.md) → [`docs/architecture/VISION.md`](docs/architecture/VISION.md) → [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.md) |
| 理解架构 | [`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.md) → [`docs/architecture/PLATFORM_KERNEL.md`](docs/architecture/PLATFORM_KERNEL.md) → [`docs/architecture/CAPABILITY_PACKAGE.md`](docs/architecture/CAPABILITY_PACKAGE.md) |
| 接入公开协议 | [`docs/protocol/PROTOCOL_V0.md`](docs/protocol/PROTOCOL_V0.md) → [`docs/spec/KERNEL_V0_ALPHA_CONTRACT.md`](docs/spec/KERNEL_V0_ALPHA_CONTRACT.md) |
| 写第一个能力包 | [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md) |
| 写 agent / 模型 / 体验包 | [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](docs/guides/AGENT_PACKAGE_AUTHORING.md)、[`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](docs/guides/MODEL_PROVIDER_INTEGRATION.md)、[`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md`](docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md) |
| 挂载第三方 Web surface | [`docs/guides/SURFACE_HOSTING.md`](docs/guides/SURFACE_HOSTING.md) |
| 看当前状态 | [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.md) |
| 看下一步 | [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.md) |

## 延后事项

下面这些方向有价值，但不属于内核——它们都将以普通能力包的形态到来：

- 兼容 SillyTavern 资源与扩展的接入项目 YdlTavern——独立仓库，跑在 Yggdrasil 之上（[`docs/tavern/TAVERN_COMPAT.md`](docs/tavern/TAVERN_COMPAT.md)）。
- 生产级长期自治 agent、多 agent 协作、生产级记忆系统、世界模拟、导演。
- 外部游戏引擎接入（UE5、Godot、Unity、Web 端）。
- 完整 Studio、ComfyUI 风格节点编辑器、市场。
- 最终视觉设计。

## 协议

Yggdrasil 以 GNU Affero General Public License v3.0（AGPLv3）发布，详见 [`LICENSE`](LICENSE)。
