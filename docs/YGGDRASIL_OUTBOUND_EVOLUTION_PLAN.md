# Yggdrasil Outbound Evolution Plan (YdlTavern Co-evolution)

> [English](./YGGDRASIL_OUTBOUND_EVOLUTION_PLAN.en.md) · [中文](./YGGDRASIL_OUTBOUND_EVOLUTION_PLAN.md)
>
> 临时文档。Yggdrasil 端配合 YdlTavern 推进的协同进化计划。每阶段完成后更新；全部完成后整体删除并合并到长期文档（`docs/protocol/PROTOCOL_V0.md`、`docs/guides/MODEL_PROVIDER_INTEGRATION.md`、`docs/ALPHA_STATUS.md` 等）。

## 立场

YdlTavern 是 Yggdrasil 的第一个真实使用者。开发 YdlTavern 的过程中暴露了 Yggdrasil 的几个真实缺口：

```text
缺口 1: HostProfile 没有 outbound.execute section（只有 outbound.git）
缺口 2: 包 manifest 没有 secret_ref 声明字段
缺口 3: kernel.outbound.execute 是 unary，没有流式（live model calls 必需）
缺口 4: 子进程 JSON-RPC 边界没有 kernel.outbound.* 客户端助手
缺口 5: docs/protocol/PROTOCOL_V0.md 没文档化 outbound methods
缺口 6: 缺乏 forge profile 模板示范 live executor 配置
```

这一轮把这些缺口一并补齐。补齐后 Yggdrasil 才真正能让 YdlTavern（以及未来其他外部包）做到"真实模型调用走平台 + secret 不直存包内 + 流式 + 审计"。

## 设计决策（已确认）

```text
- 流式作为 kernel 能力之一：新增 kernel.outbound.stream 公开协议方法
- secret_refs 在 manifest 声明：fail-closed 拒绝未声明的 secret 引用
- live HTTP outbound 仍然 opt-in：默认 deny_all，profile 显式启用
- 子进程通过 SDK helper 调 kernel.outbound.*：不让 YdlTavern 端手写 JSON-RPC
- 不污染内核命名空间：不增加 chat/turn/agent 概念
- 不为 YdlTavern 加特殊路径：所有改动是通用底座
```

## 阶段总览

| 阶段 | 内容 | 依赖 |
|---|---|---|
| Y0 | 计划提交 | - |
| Y1 | HostProfile.outbound.execute schema + executor builder | - |
| Y2 | manifest permissions.secret_refs 声明 + 校验 | - |
| Y3 | kernel.outbound.stream 协议方法 + LiveHttpStreamingExecutor | Y1, Y2 |
| Y4 | 子进程 SDK kernel.outbound.* 助手 | Y3 |
| Y5 | forge-with-live-models.example.yaml + conformance 全套 | Y1-Y4 |
| Y6 | 文档收敛（PROTOCOL_V0 / MODEL_PROVIDER_INTEGRATION / ALPHA_STATUS） | Y1-Y5 |

## Y1: HostProfile.outbound.execute schema

**位置**: `crates/ygg-cli/src/cli.rs`、`crates/ygg-cli/src/commands/host.rs`

### 改动

```text
HostProfile YAML 新增 section:

outbound:
  git:                                  # 已有
    enabled: true
    executor: real
    allowed_hosts: [...]
    ...
  execute:                              # 新增
    enabled: false                      # 默认 false
    executor: deny_all                  # deny_all | fake | live
    allowed_hosts: []                   # 精确 host 或 *.wildcard
    https_only: true
    timeout_ms: 30000
    allow_redirects: false
    allow_insecure_loopback_for_tests: false

Rust 端:
  HostProfile 结构体新增 outbound.execute 字段
  build_outbound_execute_executor(config) -> Box<dyn OutboundExecutor>
    根据 executor 字段选 DenyAll / Fake / LiveHttp
  Runtime::with_outbound_executor 在 host serve 启动时注入

LiveHttpOutboundExecutor::new_from_profile(config) 新构造器
```

### 验证

```text
cargo check -p ygg-cli
cargo test -p ygg-cli (新增 host_profile_execute_*)
cargo test -p ygg-runtime (验证 executor 注入正确)
默认 forge-alpha.yaml 不配 execute（保持 deny_all 默认行为）
```

## Y2: manifest permissions.secret_refs 声明

**位置**: `crates/ygg-core/src/manifest.rs`、`crates/ygg-runtime/src/runtime/protocol_dispatch.rs`

### 改动

```text
manifest.yaml 新字段:

permissions:
  network:
    declarations:
      - host: api.openai.com
        methods: [POST]
  secret_refs:                          # 新增
    - secret_ref:env:OPENAI_API_KEY
    - secret_ref:env:DEEPSEEK_API_KEY

Rust 端:
  PermissionsManifest::secret_refs: Vec<String>
  解析时校验每个 secret_ref 形式合法（复用 secret.rs::parse_secret_ref）

dispatch_outbound_execute / _stream:
  对 secret_headers 中每个 secret_ref:
    检查是否在 caller package manifest.permissions.secret_refs 列表内
    未声明 → ProtocolError code=permission_denied, fail-closed

ygg conformance:
  outbound_secret_ref_undeclared_fails
  outbound_secret_ref_declared_resolves
```

### 边界

```text
现有 packages/official/* 都不用 secret_ref → 不需要改 manifest
向后兼容：缺失 secret_refs 字段视为空数组
```

### 验证

```text
cargo test -p ygg-core (manifest 解析 + 校验)
cargo test -p ygg-runtime (dispatch 端 fail-closed)
ygg conformance 新增 cases pass
```

## Y3: kernel.outbound.stream 协议方法

**位置**: `crates/ygg-runtime/src/protocol.rs`、`crates/ygg-runtime/src/runtime/outbound.rs`、`crates/ygg-runtime/src/runtime/protocol_dispatch.rs`、`crates/ygg-runtime/src/runtime/streaming.rs`

### 设计

```text
新增公开协议方法:
  kernel.outbound.stream
    params: 同 kernel.outbound.execute, 加 stream_options { buffer_size, frame_format }
    return: { stream_id: uuid }
    然后通过 kernel/stream.* 事件推送帧

帧 envelope（复用 kernel/stream.* 已有事件）:
  kernel/stream.started   { stream_id, capability_id, executor_kind, redaction_state }
  kernel/stream.chunk     { stream_id, frame_index, chunk_shape, redaction_state }
                          chunk_shape 是 redacted - 不含 raw secret
                          但提供给订阅 subprocess 的 raw_chunk frame 通过传输层（详见下）
  kernel/stream.ended     { stream_id, status, usage, cost, redaction_state }
  kernel/stream.error     { stream_id, code, message }
  kernel/stream.cancelled { stream_id, reason }
  kernel/stream.timeout   { stream_id, timeout_ms }

OutboundExecutor trait 扩展:
  fn execute_stream(&self, request: OutboundExecutorRequest, sink: StreamSink) -> Result<(), OutboundError>
    DenyAllOutboundExecutor: 拒绝
    FakeOutboundExecutor: emit deterministic chunks（用于测试）
    LiveHttpOutboundExecutor: reqwest streaming + tokio channel forward

关于 raw chunks 传给 subprocess:
  方案 A: 通过 kernel/stream.chunk 事件直接 push raw bytes（events 中允许 chunk_shape 是 redacted text or base64）
  方案 B: 通过 separate raw stream channel（subprocess SDK 订阅）
  推荐方案 A：简单，复用已有 stream subsystem，redaction policy 应用在 chunk_shape 上
  对于 SSE 文本流，chunk_shape 包含解码后的 SSE 行（已 redact secret patterns）

Cancel/timeout:
  kernel.capability.cancel 接 stream_id（已存在）
  per-stream timeout 触发 stream.timeout + emit ended
```

### 改动

```text
crates/ygg-runtime/src/protocol.rs
  KernelMethod::OutboundStream 新增
  参数 schema 注册到 ProtocolRegistry

crates/ygg-runtime/src/runtime/protocol_dispatch.rs
  fn dispatch_outbound_stream
  权限校验同 outbound_execute
  secret_ref 声明检查（Y2）
  允许 streaming executor（拒绝 deny_all）

crates/ygg-runtime/src/runtime/outbound.rs
  trait OutboundExecutor::execute_stream（默认 unsupported）
  StreamSink trait
  LiveHttpStreamingExecutor 实现（reqwest::Response::bytes_stream）
  FakeStreamingExecutor 实现

crates/ygg-runtime/src/runtime/streaming.rs
  outbound stream 整合到 capability stream lifecycle
  stream_id 复用同一 uuid 命名空间
  redaction policy 应用到 chunk_shape

docs:
  PROTOCOL_V0 新增 outbound.stream 章节
  MODEL_PROVIDER_INTEGRATION 新增 streaming 章节
```

### 验证

```text
cargo test -p ygg-runtime (outbound_stream_* 单元测试)
ygg conformance:
  outbound_stream_lifecycle_started_chunk_ended
  outbound_stream_cancel_emits_cancelled
  outbound_stream_timeout_emits_timeout
  outbound_stream_fake_emits_deterministic_chunks
  outbound_stream_secret_ref_undeclared_fails
  outbound_stream_executor_kind_audited
```

## Y4: 子进程 SDK kernel.outbound.* 助手

**位置**: `sdk/typescript/subprocess/`、`crates/ygg-runtime/src/subprocess.rs`

### 设计

```text
现状:
  subprocess SDK 只能响应 capability.invoke
  没有发起 kernel.outbound.* 的路径

目标:
  subprocess SDK 提供 kernelClient
    sendKernelRequest<T>(method: string, params: unknown): Promise<T>
    streamKernelRequest(method: string, params: unknown, callbacks: { onChunk, onEnd, onError, onCancelled, onTimeout }): { cancel: () => void }

  内部实现:
    通过 stdout 发出 JSON-RPC envelope（method 是 kernel.* 而非 capability.invoke）
    runtime 端 subprocess.rs 识别这是子进程发起的反向调用
    转发到 ProtocolRegistry::dispatch（带 caller principal = subprocess package）
    response 通过 stdin 发回（带匹配的 id）
    streaming chunks 通过 stdin 推送（kernel/stream.chunk events）

边界:
  caller principal 严格绑定到 subprocess 的 package_id
  不允许 spoof 其他 package
  permission/secret_ref 校验同公开协议路径
```

### 改动

```text
sdk/typescript/subprocess/src/
  kernel-client.ts (新增)
    KernelClient 类
    sendKernelRequest / streamKernelRequest
    内部 id 生成、pending Promise 管理、stdin 解析

  outbound.ts (新增)
    executeOutbound(params): Promise<OutboundResponse>
    streamOutbound(params, callbacks): { cancel }
    包装 kernel-client，类型化输入输出

  index.ts 导出 KernelClient / outbound helpers

crates/ygg-runtime/src/subprocess.rs
  解析子进程 stdout JSON-RPC：
    method == 'capability.invoke' → 现有逻辑
    method.starts_with('kernel.') → 转发到 ProtocolRegistry::dispatch
    response 写回 stdin，带 caller principal=subprocess package
    streaming events 写回 stdin（kernel/stream.* envelopes）
```

### 验证

```text
sdk/typescript/subprocess typecheck/build
新增 examples/packages/subprocess-outbound-canary/
  manifest 声明 network + secret_refs（用 fake env var）
  capability 通过 SDK 调 kernel.outbound.execute (FakeExecutor 路径)
  capability 通过 SDK 调 kernel.outbound.stream（验证 chunks 收到）
  cancel 路径验证

cargo test -p ygg-runtime subprocess_outbound_*
ygg conformance subprocess_outbound_through_kernel_*
```

## Y5: forge-with-live-models.example.yaml + conformance

**位置**: `profiles/`、`crates/ygg-cli/src/conformance/`

### 改动

```text
profiles/forge-with-live-models.example.yaml （新增）:
  说明这是 example，不是默认 profile
  完整 outbound.execute live 配置
  注释指引：要真用必须设置 OPENAI_API_KEY 等环境变量
                  + YGG_LIVE_MODEL_TESTS=1 才会跑 live smoke tests

ygg conformance 新增类:
  outbound_execute_*
  outbound_stream_*
  manifest_secret_refs_*
  subprocess_outbound_*

预期总 case 数: 现 329 + ~12 → ~341
```

### 验证

```text
cargo test --workspace
ygg conformance（所有 case pass，包括默认仍 deny_all）
forge-with-live-models.example.yaml 通过 schema 校验
真实 live smoke test 标记为 opt-in（YGG_LIVE_MODEL_TESTS=1 + 环境变量）
默认 CI 仍不联网
```

## Y6: 文档收敛

```text
docs/protocol/PROTOCOL_V0.md / .en.md
  outbound 段：execute / stream / git_fetch
  完整 envelope 示例
  错误码表
  redaction state 枚举

docs/guides/MODEL_PROVIDER_INTEGRATION.md / .en.md
  host profile 配置示例
  manifest 声明示例（network + secret_refs）
  subprocess SDK 使用示例
  streaming 章节
  YGG_LIVE_MODEL_TESTS opt-in 模式

docs/ALPHA_STATUS.md / .en.md
  outbound stream + manifest secret_refs 列入完成清单

CONFORMANCE_MATRIX 更新
NEXT_STEPS 移除已完成项
```

## 边界（贯穿全程）

```text
不增加内核内容概念（chat/turn/agent 等不进 kernel）
不为 YdlTavern 走捷径 - 所有改动通用
不让默认 profile 联网 - opt-in 严格
不存 raw secret - 全 secret_ref + host resolver
不绕过 audit / redaction
不改向后兼容 manifest（缺失 secret_refs 视为空数组）
不引入 unwrap()/panic!() - 错误返回 Result
```

## 完成判据

```text
每阶段:
  cargo check / test 通过
  ygg conformance 新增 case 全 pass
  现有 case 0 regression
  commit + push origin/main

总完成:
  Y1-Y6 全 push
  forge-with-live-models.example.yaml 可加载
  默认 forge-alpha.yaml 行为不变
  文档同步
  YdlTavern 端 P3.5 能挂上去
```

## 与 YdlTavern 计划的耦合点

```text
YdlTavern P3.5 (model.live_call) 依赖:
  Y1 (HostProfile.outbound.execute) - 必须先
  Y2 (manifest.secret_refs)         - 必须先
  Y3 (kernel.outbound.stream)       - streaming 路径必须
  Y4 (subprocess SDK helper)        - YdlTavern engine 调用面
  Y5 (example profile)              - 用户配置参考

YdlTavern P1 (Golden Harness) 与 Yggdrasil 无依赖 - 可完全并行

YdlTavern P2 (Tokenizer) 与 Yggdrasil 无依赖 - 可完全并行
  但 P2.5 验证依赖 P1，与 Y* 仍无关
```
