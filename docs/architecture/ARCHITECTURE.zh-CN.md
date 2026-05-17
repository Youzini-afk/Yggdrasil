# 架构

> [English](./ARCHITECTURE.md) · [中文](./ARCHITECTURE.zh-CN.md)

Yggdrasil 有两个架构层：一个承载能力包的内核，以及能力包本身。内核很小，内容无关。一切有意义的东西都生活在能力包里。

```text
┌─────────────────────────────────────────────────────────────────┐
│ Capability Packages (every meaningful concept lives here)        │
│                                                                  │
│   official packages          third-party packages                │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────┐ │
│   │ conversation │ │ tavern compat│ │ world sim    │ │  ...   │ │
│   │ runtime      │ │ (future)     │ │ (community)  │ │        │ │
│   └──────────────┘ └──────────────┘ └──────────────┘ └────────┘ │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────┐ │
│   │ memory pack  │ │ agent pack   │ │ inspector ui │ │  ...   │ │
│   └──────────────┘ └──────────────┘ └──────────────┘ └────────┘ │
│                                                                  │
│   no privilege difference between official and third-party       │
└─────────────────────────────────────────────────────────────────┘
                          ▲    same contract    ▲
                          │                     │
┌─────────────────────────────────────────────────────────────────┐
│ Yggdrasil Kernel (content-free)                                  │
│                                                                  │
│   sessions      events       packages       capabilities         │
│   permissions   sandbox      hooks          assets               │
│                                                                  │
│   schemas, IDs, ordering, replay, transports                     │
└─────────────────────────────────────────────────────────────────┘
                          ▲    public protocol    ▲
                          │                       │
┌─────────────────────────────────────────────────────────────────┐
│ Transports                                                       │
│   in-process • stdio JSON-RPC • TCP JSON-RPC • HTTP • WebSocket  │
│   (WASM host • remote endpoint)                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 两个层

### 内核

内核只承载能力包，不干别的。完整职责清单见 `PLATFORM_KERNEL.md`。简言之：身份、session、不透明事件日志、能力包注册、capability fabric、扩展点分发、权限、transport。

### 能力包

能力包提供平台上每一个有意义的概念：角色、提示词、模型、agent、世界、规则、记忆、呈现，一切。详见 `CAPABILITY_PACKAGE.md`。

能力包可以是 Rust in-process、subprocess、WASM 或 remote。内核对四种一视同仁。

## 边界规则

这些不是偏好。这些是不变量。

### 1. 内核对内容一无所知

角色、场景、世界、提示词、模型、轮次、聊天、agent、记忆、游戏、规则、骰子、物品栏、类型——内核里没有这些。如果一个概念对创作者或玩家有意义，它就活在能力包里。

### 2. 官方包没有特权

官方包能做的，第三方包都能做。同一个 manifest、同一套 fabric、同一个 hook、同一道权限闸门。不存在基于 package id 的内核捷径。

### 3. 协议优先

内核暴露一份公开契约。Studio、CLI、in-process 能力包、subprocess 能力包、WASM 能力包和 remote 服务使用同一份契约。没有私有旁路。

### 4. 多种 entry 形式，地位平等

能力包可以是 `rust_inproc`、`subprocess`、`wasm` 或 `remote`。打包形式是实现细节。fabric 对它们一视同仁。

### 5. 事件是真相，但对内核不透明

内核为事件排序并持久化。它不解释 payload。意义属于能力包。

### 6. 声明式沙箱

副作用、网络可达性、文件系统可达性、跨能力包调用——全在 manifest 中声明。内核负责执行。未声明的效果即为违规。

### 7. 组合优于容纳

多个能力包可以共存于一个 session。不存在规范的「主体验」。冲突由 host 配置的优先级解决，而非内核默认值。

## 这张图里没有的东西

Tavern 不是内核层。它将是未来的能力包家族。

pi 不是内核层。它将以能力包的形式发布。

Studio 不是内核层。它是公开协议的客户端，和其他客户端一样。它可能以官方包加 UI shell 的形式发布。

外部游戏引擎不是内核层。它们以 remote-entry 能力包或协议客户端的身份参与。

## 仓库地图

Yggdrasil Foundation Alpha 工作区：

```text
crates/ygg-core      kernel types: ids, schemas, manifests, principals, opaque events
crates/ygg-runtime   kernel scheduler: sessions, packages, capabilities, hooks, surfaces,
                     proposals, assets, branches, projections, sandbox, transports
crates/ygg-service   public protocol surface (HTTP /rpc, SSE event subscribe)
crates/ygg-cli       host modes, manifest tools, package authoring, conformance
clients/web          public-protocol Home/Play, Forge, and Assist shell
packages/official    foundation capability packages loaded through ordinary manifests
sdk/typescript       subprocess-package authoring helpers and template runtime
profiles/            host profiles for autoloading sets of packages
examples/            example package manifests and fixtures
```

内核 crate 是内容无关的。对话、世界、agent、记忆和模型行为——在加入时——以普通能力包的形式到来，不享受内核特权。

## 如何阅读其余文档

- `CHARTER.md` 讲原则。
- `PLATFORM_KERNEL.md` 讲内核做什么、不做什么。
- `CAPABILITY_PACKAGE.md` 讲能力包契约。
- `EXTENSION_POINTS.md` 讲 hook 契约。
- `EVENT_MODEL.md` 讲不透明事件日志。
- `RUNTIME_LIFECYCLE.md` 讲内核侧生命周期。
- `protocol/PROTOCOL_V0.md` 讲公开协议。
