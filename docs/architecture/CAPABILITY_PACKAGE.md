# 能力包规范

> [English](./CAPABILITY_PACKAGE.en.md) · [中文](./CAPABILITY_PACKAGE.md)

能力包是 Yggdrasil 的分发和执行单元。只要不属于内核，就以能力包发布。

本文档说明能力包如何描述自身、如何加载，以及如何与内核和其他能力包交互。无论来源如何，每个能力包都遵守同一套规则。

## 平等规则

官方包、第三方包、in-process 包、子进程包、WASM 包和远端包共享同一份清单格式、同一个生命周期、同一套能力织物和同一个权限系统。

没有私有 API。官方包能做的，任何能力包都能做。

## 清单

能力包由清单描述。清单是一份可序列化文档，必须符合已发布的 schema。

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

内核拒绝未通过 schema 验证的清单。若能力包请求的权限超出 host 策略，内核也会拒绝加载。

## Entry 形式

四种入口形式都是一等的。选择哪一种，是实现细节。

### rust_inproc

以 Rust crate 或共享库加载，并编译到内核能力包 ABI。它速度快，没有 IPC 成本，性能最好。信任级别最高。崩溃可能影响 host；沙箱就是 host 本身。

### subprocess

内核启动子进程，通过 stdio 或本地 socket 上的 JSON-RPC 通信。它与语言无关，崩溃会被隔离。性能受 IPC 限制。

### wasm

内核在 WASM host 内运行能力包，并使用声明的内存和 CPU 上限。隔离性强。可用语言受限于能否编译到 WASM，性能受 WASM 和 ABI 编组影响。

### remote

能力包可以运行在任何能通过 HTTP 或 WebSocket 访问的位置。连接需要认证。这适合托管服务，也适合以能力包身份接入的外部系统。

能力包可以声明备用入口。例如同时提供 `rust_inproc` 和 `subprocess`，由 host 按策略选择。

## 生命周期

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

每次状态转换都会发出内核事件。

## 能力契约

一个能力由 `id` 和 `version` 标识。调用由 `input_schema` 和 `output_schema` 约束类型。能力可以支持流式输出。

消费方用 id 和版本约束请求能力。内核根据以下条件选择 provider：

1. 会话作用域内的活跃能力包集合。
2. 会话/profile 中声明的优先级规则。内核没有默认优先级；策略由 host 或路由能力包配置。
3. 版本兼容性。

不存在隐式的「官方包优先」规则。

如果两个能力包提供同一个能力 id，而 host 未配置优先级，内核会报告 ambiguous-route 错误并拒绝调用。

## 钩子契约

能力包可以订阅扩展点。扩展点可以由内核定义，也可以由能力包定义。订阅会声明时机，以及处理器能否修改或否决。

内核按声明的语义分发钩子。订阅方按声明顺序运行；若顺序相同，则使用 host 配置的订阅方优先级。

详见 `EXTENSION_POINTS.md` 了解内核发出的扩展点集合及契约。

## 权限与沙箱

清单是能力包与 host 的契约。内核会在每次操作上强制执行：

- 未声明的事件追加被拒绝。
- 未声明的网络调用被拒绝。
- 未声明的跨能力包调用被拒绝。
- 超出其声明 `side_effects` 的能力被拒绝。

host 可以继续叠加额外策略，例如 deny-list、配额和审计。能力包无法绕过。

## 分发

能力包分发包含清单和入口产物：

- `rust_inproc`：源 crate 或与 host ABI 版本匹配的预编译 `cdylib`。
- `subprocess`：目标平台的可执行文件加清单。
- `wasm`：`.wasm` 模块加清单。
- `remote`：仅有带 endpoint 的清单。

能力包注册表不在内核范围内。Host 和工具可以在此之上构建注册表。

## 版本管理

`version` 遵循 semver。清单格式的 `schema_version` 与能力包版本无关。

`rust_inproc` 的破坏性 ABI 变更通过入口中的新 `abi_version` 标识。Host 拒绝加载 ABI 版本不匹配的能力包。

## 身份

能力包 id 带命名空间。内核不拥有命名空间；约定和注册表拥有。

内核只在单个 host 实例内强制唯一性。
