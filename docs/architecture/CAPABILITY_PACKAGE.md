# 能力包规范

> [English](./CAPABILITY_PACKAGE.en.md) · [中文](./CAPABILITY_PACKAGE.md)

能力包是 Yggdrasil 上的分发和执行单元。平台上每一个不属于内核的概念都以能力包的形式发布。

本文档规定能力包如何描述自身、如何加载、如何与内核及其他能力包交互，以及每份能力包——无论来源——必须遵守的规则。

## 平等规则

官方包、第三方包、in-process 包、subprocess 包、WASM 包和 remote 包共享同一个 manifest、同一个 lifecycle、同一套 capability fabric 和同一个权限系统。

没有私有 API。官方包能做的，任何能力包都能做。

## Manifest

能力包由 manifest 描述。这是一份符合已发布 schema 的可序列化文档。

```yaml
schema_version: 1

id: org/name              # globally unique, namespaced
version: 0.1.0            # semver
display_name: ...
description: ...
author: ...
license: ...

entry:
  kind: rust_inproc | subprocess | wasm | remote

  # kind: rust_inproc
  crate: path or registry coordinate
  symbol: register_fn
  abi_version: 1

  # kind: subprocess
  command: [executable, args...]
  env: { ... }
  transport: jsonrpc-stdio | jsonrpc-tcp

  # kind: wasm
  module: path or url
  abi_version: 1
  memory_limit_mb: 64

  # kind: remote
  endpoint: https://... or wss://...
  auth: { scheme: bearer | mtls | none, ... }

provides:
  - id: org/name/capability
    version: 0.1.0
    input_schema: <jsonschema or ref>
    output_schema: <jsonschema or ref>
    streaming: false
    side_effects: [event_append, network, filesystem, package_call, ...]
    description: ...

consumes:
  - id: other-org/cap
    version: ^0.2

contributes:
  schemas:
    - id: org/name/event/foo
      schema: <jsonschema>
  hooks:
    - extension_point: kernel/event.after_append
      handler: handle_event
      timing: async
  assets:
    - id: org/name/asset/...
      mime: ...
      source: ...
  extension_points:
    - id: org/name/lifecycle.before_step
      payload_schema: <jsonschema>
      timing: sync | async
      modifiable: true
      short_circuit: true
  surfaces:
    - id: org/name/entry
      version: 0.1.0
      slot: experience_entry        # | home_card | play_renderer | forge_panel | asset_editor | assistant_action
      title: ...
      description: ...
      capability_id: org/name/launch
      activation:
        launch_capability_id: org/name/launch
        session_template:
          labels: [...]
          metadata: { ... }
        input_schema: <jsonschema>
      required_permissions:
        - permission: events.read
          scope: session
          reason: render the play surface
          risk: low                 # | medium | high
      approval_policy: none         # | user_approval | fork_then_approve
      metadata: { ... }

permissions:
  network:
    hosts: [api.example.com] | none | any
  filesystem:
    paths: [./data] | none
  events:
    append: true
    read: true
  packages:
    call: [other-org/*]
  declared_side_effects: [user-data-read, llm-inference, ...]

sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 128
  wall_clock_ms: 30000
```

内核拒绝未通过 schema 验证的 manifest，并拒绝加载请求超出 host 策略权限的能力包。

## Entry 形式

四种都是一等的。选择是实现细节。

### rust_inproc

以编译至内核能力包 ABI 的 Rust crate 或共享库加载。快速、无 IPC、满性能。Trust level：最高。崩溃可能影响 host；沙箱即 host 本身。

### subprocess

内核启动子进程，通过 stdio 或本地 socket 上的 JSON-RPC 通信。语言无关。崩溃被隔离。性能受 IPC 限制。

### wasm

内核在 WASM host 内运行能力包，带有声明的内存和 CPU 上限。强隔离。语言灵活性受限于可编译为 WASM 的语言。性能受 WASM 和 ABI 编组限制。

### remote

能力包运行在任何可通过 HTTP 或 WebSocket 访问的地方。经过认证。适用于托管服务和作为能力包参与的外部系统。

能力包可以声明替代 entry（例如 `rust_inproc` 加 `subprocess` 备选），让 host 按策略选择。

## Lifecycle

```text
discovered  -> kernel sees the manifest
loading     -> manifest validated, sandbox prepared
starting    -> entry point booted, kernel handshake
ready       -> capabilities and hooks registered, accepting calls
degraded    -> reachable but reporting reduced ability
stopping    -> graceful shutdown signal sent
stopped     -> resources released
unloaded    -> manifest no longer active in the host
```

状态转换发出内核事件。

## Capability 契约

一个 capability 由 `id` 和 `version` 标识。调用由 `input_schema` 和 `output_schema` 约束类型。它们可以 streaming。

消费方通过 id 加版本约束请求 capability。内核基于以下条件选择 provider：

1. session 作用域内的活跃能力包集合。
2. session/profile 中声明的优先级规则（而非内核默认值；优先级策略本身由 host 或路由能力包配置）。
3. 版本兼容性。

不存在隐式的「官方包优先」规则。

如果两个能力包提供同一个 capability id，而 host 未配置优先级，内核报告 ambiguous-route 错误并拒绝调用。

## Hook 契约

能力包可以订阅扩展点（内核定义或能力包定义）。订阅声明时机以及 handler 是否可以修改或 veto。

内核按声明的语义分发 hook。订阅方按声明顺序运行；平局由 host 配置的订阅方优先级打破。

详见 `EXTENSION_POINTS.md` 了解内核发出的扩展点集合及契约。

## 权限与沙箱

manifest 是与 host 的契约。内核在每次操作上强制执行：

- 未声明的事件追加被拒绝。
- 未声明的网络调用被拒绝。
- 未声明的跨能力包调用被拒绝。
- 超出其声明 `side_effects` 的 capability 被拒绝。

host 可以在上层叠加额外策略（deny-list、配额、审计）。能力包无法绕过。

## 分发

能力包分发包含 manifest 和 entry 产物：

- `rust_inproc`：源 crate 或与 host ABI 版本匹配的预编译 `cdylib`。
- `subprocess`：目标平台的可执行文件加 manifest。
- `wasm`：`.wasm` 模块加 manifest。
- `remote`：仅有带 endpoint 的 manifest。

能力包注册表不在内核范围内。Host 和工具可以在此之上构建注册表。

## 版本管理

`version` 遵循 semver。manifest 格式的 `schema_version` 与能力包版本无关。

`rust_inproc` 中的破坏性 ABI 变更通过 entry 中的新 `abi_version` 标识。Host 拒绝加载 ABI 版本不匹配的能力包。

## 身份

能力包 id 是带命名空间的。内核不拥有 namespace；约定和注册表拥有。

内核仅在一个 host 实例内强制唯一性。
