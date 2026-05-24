# 架构

> [English](./ARCHITECTURE.en.md) · [中文](./ARCHITECTURE.md)

Yggdrasil 分三层：内容无关的内核、可复用的能力包、以及使用能力包的项目。内核小、对内容一无所知；一切有意义的东西都生活在能力包里，项目是 host / runtime 管理的实例，不是内核 ontology。

```text
┌─────────────────────────────────────────────────────────────────┐
│ 项目（Home 上的可启动实例：YdlTavern / coding agent / ...）             │
└─────────────────────────────────────────────────────────────────┘
                          ▲     使用能力包     ▲
                          │                    │
┌─────────────────────────────────────────────────────────────────┐
│ 能力包（所有有意义的概念都长在这里）                                │
│                                                                  │
│   官方包                  第三方包                                 │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────┐ │
│   │ 对话运行时    │ │ Tavern 兼容  │ │ 世界模拟      │ │  ...   │ │
│   │              │ │ （未来）      │ │ （社区）      │ │        │ │
│   └──────────────┘ └──────────────┘ └──────────────┘ └────────┘ │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────┐ │
│   │ 记忆包       │ │ agent 包     │ │ 审查 UI       │ │  ...   │ │
│   └──────────────┘ └──────────────┘ └──────────────┘ └────────┘ │
│                                                                  │
│   官方包与第三方包之间没有特权差异                                  │
└─────────────────────────────────────────────────────────────────┘
                          ▲     同一份契约     ▲
                          │                    │
┌─────────────────────────────────────────────────────────────────┐
│ Yggdrasil 内核（对内容一无所知）                                   │
│                                                                  │
│   会话         事件         能力包        能力                     │
│   权限         沙箱         钩子          资产                     │
│                                                                  │
│   schema、id、排序、回放、传输                                     │
└─────────────────────────────────────────────────────────────────┘
                          ▲    公开协议     ▲
                          │                  │
┌─────────────────────────────────────────────────────────────────┐
│ 传输层                                                            │
│   in-process • stdio JSON-RPC • TCP JSON-RPC • HTTP • WebSocket  │
│   （WASM 宿主 • 远程端点）                                         │
└─────────────────────────────────────────────────────────────────┘
```

## 三层

### 内核

内核只承载能力包，不干别的。完整职责见 [`PLATFORM_KERNEL.md`](PLATFORM_KERNEL.md)。简言之：身份、会话、不透明事件日志、能力包注册、能力路由、扩展点分发、权限、传输。

### 能力包

能力包提供平台上每一个有意义的概念：角色、提示词、模型、agent、世界、规则、记忆、呈现，凡此种种。详见 [`CAPABILITY_PACKAGE.md`](CAPABILITY_PACKAGE.md)。

能力包可以是 Rust in-process、子进程、WASM 或远程服务。这四种入口在内核眼里没有差别。

### 项目

项目是运行时实例：有入口 surface、有自己的状态和数据目录，可以在 Home 上显示为卡片并独立启动/停止。项目引用能力包，但不把项目概念写进内核。Host 负责 `ProjectDescriptor`、`ProjectRegistry`、per-project 数据目录、项目级 secret store 和 Home 生命周期。详见 [`../guides/PROJECT_MODEL.md`](../guides/PROJECT_MODEL.md)。

## 边界规则

下面这些不是偏好，是不变量。

### 1. 内核对内容一无所知

角色、场景、世界、提示词、模型、回合、聊天、agent、记忆、游戏、规则、骰子、背包、题材——内核里没有这些。一个概念只要对玩家或创作者有意义，就属于能力包。

### 2. 官方包没有特权

官方包能做的，第三方包都能做。同一份清单、同一套机制、同一组钩子、同一道权限闸门。没有按包名走的内核捷径。

### 3. 协议优先

内核暴露一份公开契约。Studio、CLI、in-process 包、子进程包、WASM 包、远程服务，全都用这一份契约。没有私有旁路。

### 4. 入口形式平等

能力包可以是 `rust_inproc`、`subprocess`、`wasm` 或 `remote`。打包形式只是实现细节，内核对四种一视同仁。

### 5. 事件是真相，但对内核不透明

内核负责给事件排序并持久化。它不解读 payload，意义由能力包给出。

### 6. 沙箱靠声明

副作用、网络可达性、文件系统可达性、跨包调用——都要写进清单。内核负责执行。未声明的副作用就是违规。

### 7. 组合优于容纳

多个能力包可以共存于同一个会话。不存在唯一的「主体验」。冲突由 host 配置的优先级解决，不靠内核默认。

## Contract v1 边界

公开平台规范是 [`../spec/KERNEL_V1_CONTRACT.md`](../spec/KERNEL_V1_CONTRACT.md)。v1 schema 位于 `../spec/v1/schemas/`，其中 methods、events 与 top-level schema 是 SDK 生成、conformance kit 和第三方实现的单一可信源。

### Capability handles

Manifest 字符串声明权限上限，运行时 capability handle 表示实际权威。内核在 package load / handshake / init 时铸造句柄，并可衰减、撤销、过期。能力调用、事件访问、出站请求和 secret 解析都应通过句柄或等价 runtime binding 表达，而不是靠包名或裸字符串获得特权。

详见 [`../guides/CAPABILITY_HANDLES.md`](../guides/CAPABILITY_HANDLES.md)。

### Bindings injection

路径 A 包（`entry.contract: "v1"`）启动时收到 bindings。Subprocess 包在 `package.handshake` 中接收 bindings 字典；Rust in-process 包通过 `KernelEnv` 初始化；WASM 与 remote 计划通过 WIT resource imports 与 SPIFFE/Biscuit token 兑换补齐。Bindings 只包含该包被授予的最小权威。

### Path B

路径 B 包（`entry.contract: "none"`）选择自包含运行。内核仍托管生命周期、捕获日志并发出事件，但不注入 v1 句柄、不强制 manifest 权限，也不把 manifest 声明转换成平台权威。Path A 与 Path B 都是一等参与方式；需要 capability invoke、network、secret 或 declared-vs-used audit 的包应选择 Path A。

详见 [`../guides/PATH_B_SELF_CONTAINED.md`](../guides/PATH_B_SELF_CONTAINED.md)。

## 项目层

项目层位于能力包之上。它把一组能力包、入口 surface、状态目录和 secret policy 组合成用户能看见、能启动、能卸载的运行时实例。Home 屏幕把项目显示为项目卡片；点 Play 通过 `kernel.v1.project.start` 请求宿主启动项目，然后导航到项目的 entry surface。

项目仍遵守内核不变量：内核不解释 YdlTavern、coding agent、image-gen 等内容形态；项目管理是 host/admin 协议和 runtime registry 的职责，普通包不能借项目协议获得额外权限。

## 这张图里没有的东西

Tavern 不是内核层。它将作为未来的能力包家族出现。

pi 不是内核层。它会以能力包形态发布。

Studio 不是内核层。它是公开协议的一个客户端，和别的客户端一样；未来可能以官方包加 UI shell 的形式发布。

外部游戏引擎不是内核层。它们以远程能力包或协议客户端的身份参与。

## 客户端 shell 与发布边界

### Web client architecture

`clients/web` 是 plain TypeScript SPA，通过 Vite 做 dev server、类型检查与 production build。它不把 React 或其他前端框架作为 shell 架构前提；Home / Play、Forge、Assist 都是公开协议客户端。

Web shell 与 host 的交互只走公开传输：HTTP `POST /rpc` 调用能力与 kernel 方法，SSE 订阅事件流。它不读取 SQLite、不导入 runtime crate，也没有针对官方能力包的私有旁路。

### SurfaceHost

第三方 Web surface bundle 由 iframe-based SurfaceHost 挂载。宿主创建 `sandbox="allow-scripts"` iframe，加载 `surface-frame.html`，再用 `postMessage` 发送 mount 指令。Surface 向宿主发 `{type: 'rpc.call'}`，宿主按显式 bridge 配置返回 `{type: 'rpc.result'}`。

默认没有 kernel access；host 必须显式接线 `hostBridge.callRpc`。Surface bundle 约定、iframe CSP、YdlTavern 示例与 v0 限制见 [`../guides/SURFACE_HOSTING.md`](../guides/SURFACE_HOSTING.md)。

### Desktop wrapper

`clients/desktop` 是 Tauri 2.x wrapper。生产模式嵌入 `clients/web/dist`，开发模式指向 Vite dev server。它是 Web shell 的桌面容器，不是第二套协议或私有 Studio。

v0 边界：desktop wrapper 不自动 spawn `ygg-cli host serve`；用户需要单独运行 host。后续可增加受控子进程管理，但仍应保持公开协议边界。构建要求见 [`../../BUILDING.md`](../../BUILDING.md)。

### Release pipeline

Release 由 `v*` tag 触发 GitHub Actions。Pipeline 构建 Web shell、构建跨平台 Tauri 安装包，并创建 draft GitHub release。版本号通过 `scripts/release-version.sh` 同步到 Cargo、Web package、desktop package 与 Tauri 配置。

当前 release 不包含签名、公证或自动更新。发布步骤见 [`../../BUILDING.md`](../../BUILDING.md)，变更记录见 [`../../CHANGELOG.md`](../../CHANGELOG.md)。

## 仓库地图

Yggdrasil Foundation 工作区：

```text
crates/ygg-core      内核类型：id、schema、清单、身份、不透明事件
crates/ygg-runtime   内核调度：会话、能力包、能力、钩子、surface、
                     提案、资产、分支、projection、沙箱、传输
crates/ygg-service   公开协议层（HTTP /rpc、SSE 事件订阅）
crates/ygg-cli       host 模式、清单工具、能力包脚手架、conformance
clients/web          Vite + plain TS 的 Home/Play、Forge、Assist shell
clients/desktop      Tauri 2.x desktop wrapper
packages/official    通过普通清单加载的官方基础能力包
sdk/typescript       子进程能力包脚手架与模板运行时
profiles/            host profile，批量自动加载能力包
examples/            示例清单与 fixture 包
```

内核 crate 对内容一无所知。对话、世界、agent、记忆、模型——一旦加入——都以普通能力包的形式到来，不享受内核特权。

## 接下来读什么

- [`CHARTER.md`](../CHARTER.md) 讲原则。
- [`PLATFORM_KERNEL.md`](PLATFORM_KERNEL.md) 讲内核做什么、不做什么。
- [`CAPABILITY_PACKAGE.md`](CAPABILITY_PACKAGE.md) 讲能力包契约。
- [`EXTENSION_POINTS.md`](EXTENSION_POINTS.md) 讲钩子契约。
- [`EVENT_MODEL.md`](EVENT_MODEL.md) 讲不透明事件日志。
- [`RUNTIME_LIFECYCLE.md`](RUNTIME_LIFECYCLE.md) 讲内核侧生命周期。
- [`../spec/KERNEL_V1_CONTRACT.md`](../spec/KERNEL_V1_CONTRACT.md) 讲公开 v1 契约与 schema。
- [`../guides/CAPABILITY_HANDLES.md`](../guides/CAPABILITY_HANDLES.md) 讲能力句柄与审计。
- [`../protocol/PROTOCOL_V0.md`](../protocol/PROTOCOL_V0.md) 讲公开协议。
- [`../guides/SURFACE_HOSTING.md`](../guides/SURFACE_HOSTING.md) 讲第三方 Web surface 托管。
- [`../guides/PROJECT_MODEL.md`](../guides/PROJECT_MODEL.md) 讲 Home 项目层与生命周期。
- [`../../BUILDING.md`](../../BUILDING.md) 讲 Web / desktop 构建与 release。
- [`../../CHANGELOG.md`](../../CHANGELOG.md) 记录发布变更。
