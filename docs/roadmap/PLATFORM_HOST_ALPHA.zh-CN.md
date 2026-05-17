# Platform Host Alpha

> [English](./PLATFORM_HOST_ALPHA.md) · [中文](./PLATFORM_HOST_ALPHA.zh-CN.md)

Platform Host Alpha 是证明 Yggdrasil 能够通过每个调用者使用的同一份公开契约托管非特权外部包的里程碑。它不是内容运行时、Studio、Tavern 兼容、pi 集成或游戏框架。

已实现切片已经就位：一个零官方包的全新 host 可以加载第三方 subprocess 包、完成 JSON-RPC-over-stdio 握手、通过公开协议暴露并调用 capability、强制执行权限/schema/超时/teardown、对已实现的 extension point 分发声明的 hook、干净地卸载，并通过 in-process 和公开传输层路径通过 hostile conformance。

后续的 **Play/Forge Surface Contract Beta** 层直接建立在此基础之上（见下方「已实现切片」）。剩余的 Host Alpha partial 项目继续在此追踪，并滚动进入 `NEXT_STEPS.md` 中的 Phase F（Foundation Alpha 收敛）。

## 里程碑定义

当一个零官方包的全新 host 可以做到以下所有时，Platform Host Alpha 完成：

1. 从其 manifest 加载第三方 subprocess 包，
2. 完成 JSON-RPC-over-stdio lifecycle 握手，
3. 通过公开协议暴露并调用 capability，
4. 强制执行包权限、namespace 所有权、schema、超时和进程 teardown，
5. 对已实现的 extension point 确定性地分发声明的 hook，
6. 干净地卸载包，
7. 通过 in-process 和公开传输层路径通过 hostile conformance。

## 已实现切片

Host Alpha 基础：

1. 协议和 principal 基础：方法信封、运行时上下文、结构化错误、package-principal 路径不存在调用者提供的包身份伪造。
2. Subprocess 包执行：JSON-RPC stdio 启动、握手、调用、调用超时、降级状态、卸载 kill。
3. 公开传输层：规范 HTTP `/rpc` 和 host JSON-RPC stdio 模式，用于非 streaming 方法。
4. Hook fabric 切片：事件追加和 capability 调用 before/after 分发、稳定排序、包拥有的 handler capability、metadata 变更、veto、卸载清理。
5. 包创作工具：Python 和 TypeScript subprocess 模板、package check、本地 fixture 运行、本地调用、package conformance。
6. 发布门槛 conformance：具名 hostile 用例，含文档矩阵覆盖。
7. 事件范围 replay 和 host-dev HTTP SSE tailing。
8. 显式 capability provider 选择，带简单版本约束。
9. 包 lifecycle 时间线、subprocess 重启、stderr 日志捕获、host diagnostics。
10. 事件日志可 rehydrate 的 asset、projection 和 session branch 底座，面向 host-dev 协议调用者。
11. 基于 profile 的 `ygg host serve`，带自动加载包、HTTP `/rpc` 和 SSE 路由。

Play/Forge Surface Contract Beta（建立在 Host Alpha 基础之上）：

12. `clients/web` 下走公开协议的 Web shell 骨架，包含 Home/Play、Forge 和 Assist surface。
13. 第一批官方基础包（`official/package-lab`、`official/schema-tools`、`official/event-tools`）是 `packages/official` 下的普通包 manifest，通过 Forge profile 自动加载。
14. 第一个 assistant 包（`official/assistant-lab`）是一个普通包，贡献 assistant action 并返回需要审批的 proposal。
15. `ygg play-create-demo` 演示了基于普通包和公开底座的第一个空白游创循环。
16. Web shell Home 路由通过公开协议发现 `experience_entry` surface、启动包支持的 session、支持 branch fork，无需官方包硬编码。
17. Web Forge 路由为包 Forge panel、proposal、asset、projection、capability 和事件 tail 提供通用公开协议检查器。
18. 包创作包括生成的 experience-surface 模板和本地 composition 描述符检查。
19. 通用 proposal lifecycle 方法（`kernel.proposal.create/get/list/approve/reject/apply`）将 assistant/包变更置于显式审批之后，并追加 `kernel/proposal.*` 审计事件。

## 剩余 Platform Host Alpha 工作

这些项目仍为 partial。它们滚动进入 Phase F 收敛和 Phase I 后台 hardening（见 `NEXT_STEPS.md`）。

1. 协议分发的 streaming 和 package-principal subscribe 权限检查。
2. 包拥有的 handler 的 hook 超时/错误审计。
3. 健康检查和超越 lifecycle 过渡事件的更丰富崩溃监控。
4. 超出每次调用显式 provider 选择的持久 provider 选择策略。
5. 超出当前核心协议分发器/service 测试的更广传输层一致性用例。
6. 超出当前轻量 subprocess 辅助/模板的更丰富 TypeScript SDK 打包。
7. 持久权限授权 rehydration 和更丰富的资源策略覆盖。

## 本里程碑的非目标

- 对话运行时，
- 模型 provider 包，
- SillyTavern 兼容，
- pi 集成，
- Studio / Prompt Inspector UI，
- 最终 UI 视觉设计或内容运行时行为，
- 游戏、世界、角色、director、记忆或 agent 语义，
- 市场或包依赖解析，
- remote 包执行，
- WASM 包执行，
- 超出显式测试的 subprocess 超时/kill 行为的完整 OS sandbox 保证。

## 必须不变量

任何官方包、客户端、service 路由或 SDK 辅助不得使用特权内核路径。官方 namespace 是普通 namespace。如果某个行为对第三方包通过公开协议和 manifest 模型不可用，则它不是 Platform Host Alpha 的功能。
