# 后续步骤

> [English](./NEXT_STEPS.md) · [中文](./NEXT_STEPS.zh-CN.md)

平台基础已经就位。Yggdrasil 现在拥有内容无关的内核、基于 manifest 的包、真正的 `rust_inproc` 和 subprocess 执行、权限/principal 系统、hook fabric 切片、surface 贡献、proposal/approval lifecycle、asset/branch/projection 底座、官方平台包、assistant 包、`official/playable-seed`、空白游创循环以及走公开协议的 Home/Play 和 Forge surface 的 Web shell。

下一个重心**不是**更多底座。而是让第一批 reference packages 足够可用，使第三方包可以在同一路径上替换它们。

## 当前位置

- Platform Foundation Alpha：已完成。
- Play/Forge Surface Contract Beta：已完成。
- First Real Capability Package Track：seed 已完成（`composition-lab`、`asset-lab`、`projection-lab`、`playable-seed`；55 个 conformance 用例）。
- Platform Host Alpha：已实现切片完成；剩余项目（streaming 分发、hook 超时审计、持久 provider 策略、更广的传输层一致性、更丰富的 SDK 打包）在 `PLATFORM_HOST_ALPHA.md` 中追踪。

详见 `docs/ALPHA_STATUS.md` 获取详细快照。

## Phase F — Foundation Alpha 收敛（已完成）

目标：停止扩大表面积。打磨粗糙边缘，锁定契约，让现有基础便于 demo、文档和扩展。

- 跨 `README.md`、`README.zh-CN.md` 和文档树刷新文档。
- 添加 `docs/product/PLAY_CREATION_MODEL.md` 以固定游创产品立场。
- 添加 `docs/ALPHA_STATUS.md` 作为已完成、partial 和 deferred 内容的活快照。
- 在代价较低处解决 Platform Host Alpha 的剩余 partial 项目。
- @oracle-led 审查轮次，检查内容形态泄漏、官方特权泄漏和 YAGNI 清理。
- 一条规范的端到端 demo 路径，有文档记录并通过 conformance 验证。

当新贡献者可以 clone 仓库、读一份 README、运行一条 host serve 命令、到达空白游创循环且没有意外时，此阶段完成。

## Phase G — Playable Experience Alpha seed（已完成）

目标：通过构建可启动、可检查、可 fork、可由 assistant 辅助的 reference packages 来证明底座，全部作为普通包实现。

这是平台第一次产出游创创作者可以坐下来体验超过一个 demo 的东西。它不是 SillyTavern，不是纯对话运行时，不是 director —— 它是最小的、诚实演练每个底座原语的体验。

这个 seed 刻意不是 canonical game runtime。`official/playable-seed` 证明 package 路径；`official/composition-lab`、`official/asset-lab` 和 `official/projection-lab` 证明周边创作与检查循环。

带入此阶段的约束：

- 内核变更是最后手段。如果体验需要新原语，先重新设计体验。
- 实现该体验的官方包必须保持可被第三方包替换。
- Assistant 必须通过 `kernel.proposal.*` 提出变更，而非通过特权路径。
- Forge 必须能够仅使用公开协议检查、fork 和编辑体验。
- Conformance 随包一起增长：至少一个 hostile 用例证明第三方体验包可以到达相同的 surface。

## Phase H — Authoring & Composition Beta+（下一步）

目标：将当前的创作切片（`init-package`、`init-composition`、`composition check`、生成的 experience 模板）转化为此仓库外的人可以用来发布包的真实创作循环。

- 更丰富的 composition 描述符（多包捆绑、可选 capability、默认激活）。
- 各 surface slot 的模板变体（play renderer、forge panel、assistant action），超越当前的 "experience template"。
- 本地开发循环体验：watch 模式、快速 reload、manifest diff、surface 预览。
- 持续扩展 `docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`，从薄 walkthrough 变成完整贡献者路径。
- 可选的包注册表形态的 surface，仍然建立在公开协议之上。

## Phase I — 底座 hardening（并行，低优先级）

作为后台工作推进，不是主角：

- 持久权限授权和更丰富的资源策略覆盖。
- 内容寻址 asset blob。
- 包拥有的 projection 执行。
- Streaming 协议分发 + package-principal subscribe 权限。
- Hook handler 超时/错误审计。
- 持久 capability provider 选择策略。
- Conformance 中更广的传输层一致性覆盖。
- WASM 和 remote 包 entry 执行。

这些项目解除特定用例的阻塞。它们不作为上述主阶段的前置条件。

## 内核范围内的无限期延后

这些仍是内核的非目标。它们可能以未来包的形式存在。

- SillyTavern 兼容 —— 见 `docs/tavern/TAVERN_COMPAT.md`。
- pi 集成 —— 见 `docs/architecture/PI_INTEGRATION.md`。
- 外部游戏引擎桥接（UE5/Godot/Unity，web 客户端）。
- 任何超出公开协议 Web shell 骨架的 UI shell、检查器或 studio。
- 记忆模型、agent loop、世界模拟、director、提示词渲染、模型 provider 抽象。
- 市场、包签名、依赖解析器。

## 如何阅读这份列表

Phase F、Phase G 的 seed 形态、Creative Capability Kit Alpha 与 Model Connectivity Kit Alpha 已经完成。Phase H 是下一步：使用官方 labs 已验证的相同接口，让第三方能力包创作与 composition 真的好用。未来 model inference 仍被推迟到 [`MODEL_INFERENCE_PREREQUISITES.md`](MODEL_INFERENCE_PREREQUISITES.md) 之后。Phase I 在后台运行，以 charter 纪律评分（无内容形态泄漏到内核，无官方特权通过任何路径泄漏）。
