# Yggdrasil

> [English](./README.md) · [中文](./README.zh-CN.md)

Yggdrasil 是一个面向 AI 原生世界、游戏、故事和游玩的扩展驱动创作平台。

它是一个内核 + 一份契约 —— 中心小、稳定、不带主观意图 —— 在它之上是一个由能力包（capability package）组成的开放生态，平台中的每一个有意义的概念都来自能力包。

## 我们为什么做这个

今天大多数 AI 原生创作工具，都把使用者切成两半：消费成品体验的玩家，和构建体验的开发者。Yggdrasil 拒绝这种切分。玩家可以审视当前 session、让 assistant 修改它、fork 它、替换其中某个能力包、再把改动反馈出去。创作者面对的是同一份公开协议、同样的能力包、同样的产品 surface。底层底座在两个方向上是完全相同的。

这一立场是内核、公开协议、官方包、Web shell 共同服务的目标。完整的产品立场见 [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.md)。

## 重心所在

- 内核只承担承载能力包的职责，仅此而已。
- 所有有意义的概念（角色、提示词、模型、agent、世界、规则、记忆等等）都由能力包提供。
- 官方包没有任何特权。同一份 manifest、同一套能力网络、同一道权限闸门。
- 创作者可以自由组合、替换或自己写新的能力包。

平台的职责是让激进的 AI 原生创作成为可能，而不是去给某条「官方路径」开特权。

## 当前状态

**Platform Foundation Alpha + Play/Forge Surface Contract Beta。**

当前底座包含：内容无关的内核、基于 manifest 的能力包系统、真正的 `rust_inproc` 与 subprocess 执行、hook fabric、SQLite 事件日志、principal 与作用域权限、surface contributions、通用 proposal/approval 生命周期、asset/branch/projection 底层、官方基础包、作为能力包存在的 assistant、空白游创循环、以及一个完全走公开协议的 Home/Play + Forge 的 Web shell。51 个具名 conformance 用例 + crate / service 单元测试覆盖整个边界。

可执行快照见 [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.md)。
后续阶段见 [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.md)。

## 仓库结构

```text
crates/
  ygg-core/      内核类型与契约，内容无关。
  ygg-runtime/   运行时主机：events、packages、capabilities、hooks、surfaces、
                 proposals、assets、branches、projections、sandbox、transports。
  ygg-service/   公开协议层（HTTP /rpc，事件 SSE 订阅）。
  ygg-cli/       host 模式、manifest 工具、能力包脚手架、conformance。
clients/
  web/           走公开协议的 Home / Play、Forge、Assist Web shell。
packages/
  official/      作为普通 manifest 加载的官方基础能力包。
sdk/
  typescript/    subprocess 能力包脚手架与模板运行时。
profiles/        host profile，用来批量自动加载能力包。
examples/        示例 manifest 与 fixture 包。
docs/            架构、协议、规范、路线图、产品、Tavern 相关文档。
```

## 快速上手

用 Forge profile 启动 host，再针对它打开 Web shell：

```bash
cargo run -p ygg-cli -- host serve \
  --http 127.0.0.1:8787 \
  --profile profiles/forge-alpha.yaml
```

另一个终端里类型检查 Web shell：

```bash
tsc -p clients/web/tsconfig.json --noEmit
```

跑完整 conformance 套件：

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

只用普通公开协议调用，跑通空白游创循环：

```bash
cargo run -p ygg-cli -- play-create-demo
```

## 常用命令

```bash
# manifest 与能力包
cargo run -p ygg-cli -- manifest validate examples/packages/echo-rust-inproc/manifest.yaml
cargo run -p ygg-cli -- package load    examples/packages/echo-rust-inproc/manifest.yaml
cargo run -p ygg-cli -- package check   examples/packages/echo-subprocess-python/manifest.yaml
cargo run -p ygg-cli -- package conformance examples/packages/echo-subprocess-python/manifest.yaml
cargo run -p ygg-cli -- capability invoke examples/packages/echo-rust-inproc/manifest.yaml \
  example/echo-rust-inproc/echo --input '{"hello":"world"}'

# 能力包脚手架
cargo run -p ygg-cli -- init-package /tmp/ygg-package        --id example/new-package        --entry subprocess --language python
cargo run -p ygg-cli -- init-package /tmp/ygg-ts-package     --id example/new-ts-package     --entry subprocess --language typescript
cargo run -p ygg-cli -- init-package /tmp/ygg-experience-pkg --id example/new-experience     --entry subprocess --language typescript-experience
cargo run -p ygg-cli -- init-composition /tmp/ygg-composition --id example/new-experience
cargo run -p ygg-cli -- composition check /tmp/ygg-composition/composition.yaml

# host 模式
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
cargo run -p ygg-cli -- host-stdio

# 验证与 demo
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
cargo run -p ygg-cli -- demo
cargo run -p ygg-cli -- sqlite-demo /tmp/ygg.db
tsc -p clients/web/tsconfig.json --noEmit
```

## 推荐先读

每篇开发文档都同时提供英文与简体中文版本，文件顶部的双语导航 blockquote 可在两种语言间切换。下面这份阅读路径覆盖了内核、能力包契约、协议、状态与路线图：

- [`docs/CHARTER.zh-CN.md`](docs/CHARTER.zh-CN.md) —— 不变的根本原则。
- [`docs/architecture/VISION.zh-CN.md`](docs/architecture/VISION.zh-CN.md) —— 平台为何而存在。
- [`docs/architecture/ARCHITECTURE.zh-CN.md`](docs/architecture/ARCHITECTURE.zh-CN.md) —— kernel + packages 两层架构。
- [`docs/architecture/PLATFORM_KERNEL.zh-CN.md`](docs/architecture/PLATFORM_KERNEL.zh-CN.md) —— 内核做什么、不做什么。
- [`docs/architecture/CAPABILITY_PACKAGE.zh-CN.md`](docs/architecture/CAPABILITY_PACKAGE.zh-CN.md) —— 能力包契约。
- [`docs/architecture/EXTENSION_POINTS.zh-CN.md`](docs/architecture/EXTENSION_POINTS.zh-CN.md) —— 扩展点 / hook 契约。
- [`docs/architecture/EVENT_MODEL.zh-CN.md`](docs/architecture/EVENT_MODEL.zh-CN.md) —— 不透明事件日志模型。
- [`docs/architecture/RUNTIME_LIFECYCLE.zh-CN.md`](docs/architecture/RUNTIME_LIFECYCLE.zh-CN.md) —— 内核侧生命周期。
- [`docs/protocol/PROTOCOL_V0.zh-CN.md`](docs/protocol/PROTOCOL_V0.zh-CN.md) —— 公开协议。
- [`docs/spec/KERNEL_V0_ALPHA_CONTRACT.zh-CN.md`](docs/spec/KERNEL_V0_ALPHA_CONTRACT.zh-CN.md) —— 可执行的 alpha 契约矩阵。
- [`docs/spec/CONFORMANCE_MATRIX.zh-CN.md`](docs/spec/CONFORMANCE_MATRIX.zh-CN.md) —— hostile conformance 路线图。
- [`docs/product/PLAY_CREATION_MODEL.zh-CN.md`](docs/product/PLAY_CREATION_MODEL.zh-CN.md) —— 游创一体的产品立场。
- [`docs/ALPHA_STATUS.zh-CN.md`](docs/ALPHA_STATUS.zh-CN.md) —— 已完成 / 部分完成 / 延后内容的实时快照。
- [`docs/roadmap/NEXT_STEPS.zh-CN.md`](docs/roadmap/NEXT_STEPS.zh-CN.md) —— 当前与下一阶段。
- [`docs/roadmap/PLATFORM_HOST_ALPHA.zh-CN.md`](docs/roadmap/PLATFORM_HOST_ALPHA.zh-CN.md) —— Host Alpha + Play/Forge Surface Beta 阶段成果。

## 延后事项

下面这些方向有价值，但不属于内核。它们都将以普通能力包的形态到来。

- SillyTavern 兼容 —— 见 [`docs/tavern/TAVERN_COMPAT.zh-CN.md`](docs/tavern/TAVERN_COMPAT.zh-CN.md)。
- pi 集成 —— 见 [`docs/architecture/PI_INTEGRATION.zh-CN.md`](docs/architecture/PI_INTEGRATION.zh-CN.md)。
- 外部游戏引擎（UE5、Godot、Unity、Web 端）—— 后续以包或 remote 入口形式接入。
- 对话运行时、模型 provider、记忆模型、agent loop、世界模拟、director。
- 最终视觉设计、完整 Studio、ComfyUI 风格节点编辑器、市场。

## 协议

Yggdrasil 以 GNU Affero General Public License v3.0（AGPLv3）发布，详见 [`LICENSE`](LICENSE)。
