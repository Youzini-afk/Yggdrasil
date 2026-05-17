# 平台内核

> [English](./PLATFORM_KERNEL.md) · [中文](./PLATFORM_KERNEL.zh-CN.md)

内核是让能力包在 Yggdrasil 上共存的最小基础设施。它很小，内容无关，并且稳定。

本文档界定内核做什么、不做什么。未被列为内核职责的一切，必须生活在能力包里。

## 内核做什么

### 1. 身份与 schema

- 为 session、event、package、capability 调用、asset 记录生成 ID。
- 在每个持久化的契约对象上维护 `schema_version`。
- 依据已发布的 schema 验证 manifest、hook 订阅和 capability 注册。

### 2. Session 外壳

- 分配和寻址 session。
- 持有每个 session 的元数据（id、created_at、label、status）。
- 承载事件流和权限作用域。
- 内核不解释 session 的用途。一个 session 就是一个带标签的事件流，附带一个能力包集合。

### 3. 只追加事件日志

- 接受已授权写入方的事件。
- 按 session 排序。
- 持久化存储。
- 按需 replay。
- 内核将事件 payload 视为不透明 JSON。意义属于能力包。

### 4. 能力包注册

- 从 manifest 加载、验证并启动能力包。
- 跟踪能力包状态（registered、loading、ready、degraded、stopped）。
- 干净地卸载。
- 调度生命周期：session 声明哪些能力包在其作用域内处于活跃状态。

### 5. Capability fabric

- 按 id 和 version 索引 capability。
- 将调用和流路由到 provider。
- 在配置时将调用记录到事件日志。
- 在消费方和提供方之间协商版本约束。

### 6. 扩展点分发

- 维护扩展点注册表。
- 持有订阅方列表。
- 按声明的顺序和时机分发 hook 调用。
- 强制超时和取消。

### 7. 权限闸门

- 识别 principal（host_admin、host_dev、package、human、assistant、anonymous）。
- 读取每个能力包的 manifest 声明权限。
- 跟踪 human 和 assistant principal 的作用域授权（`events.read`、`capabilities.invoke` 等）。
- 在事件写入、capability 调用、跨能力包调用、网络/文件系统访问上执行以上全部。
- 拒绝未声明的副作用并写入 `kernel/permission.denied` 审计事件。

### 8. Surface contributions

- 接受能力包在 manifest 中声明的 UI surface 描述符（slot 包括 `experience_entry`、`home_card`、`play_renderer`、`forge_panel`、`asset_editor`、`assistant_action`）。
- 通过公开协议暴露它们，使任何客户端都可以发现哪些是可启动的、可查看的、可让 assistant 操作的。
- 仅存储描述符。渲染和内容语义属于能力包和客户端。

### 9. Proposal 生命周期

- 调度通用的、需要审批的变更 proposal（`create`、`get`、`list`、`approve`、`reject`、`apply`）。
- 仅 apply 内核已理解的通用操作（`asset.put`、`projection.rebuild`）。
- 为每次状态转换发出 `kernel/proposal.*` 审计事件。
- 拒绝未获审批的 proposal 的 apply，或其声明的操作不被支持的 proposal。内核绝不发明领域相关的 proposal 语义。

### 10. Asset、branch 和 projection

- 维护不透明的 asset 注册表（`id`、`mime`、`hash`、`size`、`origin_package`、`metadata`、content blob）。
- 以内核记录的形式跟踪 session fork/branch 沿革。
- 维护通用的 projection 记录，通过过滤事件日志重建；内核不解释 projection 状态。
- 以上三者均可从持久事件日志恢复。

### 11. Transport 层

- 在以下通道上承载规范协议信封：in-process Rust API、HTTP `/rpc`、host JSON-RPC stdio（`ygg host-stdio`）以及 SSE 事件订阅。
- 基于配置文件的 `ygg host serve` 自动加载能力包并暴露同一 dispatcher。
- WebSocket 和 TCP transport 保留供后续工作。
- 所有 transport 呈现同一个概念协议；官方包和客户端与第三方使用同一协议。

### 12. 沙箱边界

- 在内核二进制内运行 in-process Rust 能力包（trust level `trusted_inproc`）。
- 通过 stdio 上的 JSON-RPC 启动和监管 subprocess 能力包，支持握手、调用超时、卸载时杀死、重启和 stderr 捕获（trust level `process_isolated`）。
- WASM（`wasm_sandbox`）和 remote（`remote_boundary`）entry 保留为一等 manifest 形式；执行延后。

### 13. 公开协议

- 以上内容的线路级契约。内核不使用私有旁路；官方包和客户端与第三方使用同一协议。

## 内核不做什么

内核对以下内容不携带任何意见。它们保留给能力包，包括官方包。

### 对话、提示词和模型

- 没有轮次、消息、prompt frame、context plan、model call、采样或 token 用量的概念。
- 没有 prompt 渲染，没有模板语言，没有 system/user/assistant 角色。
- 没有 model provider 抽象，没有 streaming chunk 格式，没有聊天历史。

### 世界、角色、场景、规则

- 没有世界模型、场景图或 actor 类型。
- 没有角色 schema，没有关系状态，没有物品栏，没有时钟。
- 没有规则引擎，没有条件/效果，没有骰子，没有战斗结算。

### 记忆

- 没有记忆分类，没有 embedding，没有检索策略。
- 没有摘要，没有 pin，没有合并策略。

### Agent 和 director

- 没有 agent loop，没有 planner，没有 director。
- 没有 proposal-and-commit 模式——除非能力包自己选择定义。

### 内容来源

- 没有 SillyTavern 解析器，没有 PNG 元数据读取器，没有角色卡 schema。
- 没有游戏引擎桥接，没有 UE5/Godot/Unity 胶水。

### 呈现

- 没有 UI，没有聊天面板，没有 inspector，没有编辑器。
- 没有主题，没有布局，没有 asset 渲染。

### 存储意见

- 没有业务表。内核需要存储事件、manifest 和 asset 记录。它不提供 ORM、查询构建器或面向内容的数据模型。

## 灰色地带

以下需要明确立场，以防漂移。

### Asset

内核维护 asset 注册表。它记录 `id`、`mime`、`hash`、`size`、`origin_package` 和 content blob。它不解析、渲染或解释 asset 内容。能力包拥有自己的格式。

### 事件排序

内核保证每个 session 内的单调排序和持久化。它不保证任何跨 session 排序、因果图或关联语义。因果关系/关联字段是由写入方提供的不透明元数据。

### 错误

内核错误覆盖：transport、权限、schema 验证、manifest、容量、能力包 lifecycle。能力包错误以不透明结构化失败的形式流过 capability 调用；内核不对它们分类。

### 默认值

内核不附带默认能力包。一个发行版可以捆绑官方包，但内核二进制本身在不加载任何 manifest 启动时，运行的是一个空平台：它接受 session、接受事件，但没有 capability 注册，不存在任何语义。

## 稳定性承诺

本文档通过显式修订来变更。新的职责需要论证其无法生活在能力包里。默认答案是「能力包，而非内核」。
