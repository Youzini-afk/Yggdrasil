# Yggdrasil Outbound WebSocket Evolution Plan

> 临时计划文件，每阶段完成后 push；全部完成后由 Z8 阶段删除并并入长期文档。

## 目的

补齐 Yggdrasil 出站协议中缺失的双向流式能力（WebSocket），完成 outbound 三件套：

| 能力 | 协议方法 | 状态 |
|------|----------|------|
| 一元 HTTPS | `kernel.outbound.execute` | Y1 |
| 单向流式（SSE/NDJSON） | `kernel.outbound.stream` | Y3 |
| 双向 WebSocket | `kernel.outbound.websocket.*` | **本轮 Z** |

直接驱动场景：

1. **OpenAI Realtime API / Gemini Live API** — 实时双向语音/文本，必须双向流
2. **Remote package entry form** — 平台四种 entry form 的最后一种真实兑现路径
3. **填补出站审计完成事件的小缺口**：当前 `kernel.outbound.*` 在 dispatch 时记录请求，但没有"完成时汇总"事件；流式与 ws 都需要

## 不做的事

- 不做语音编解码（PCM/Opus/G.711 解析归调用方）
- 不做 WebRTC / DataChannel（不属于 outbound 范围）
- 不做远程包真实加载（独立 entry-form 工作）
- 不替代 SSE，二者并存
- 不允许默认 `ws://`（明文）— 仅 `wss://` 默认放行
- 不在 audit 中存任何帧 payload，仅形状/字节数/seq

## 设计原则

- **复用既有约束**：HTTPS-only → wss-only；host 严格匹配；secret_ref 必须 manifest 声明（Y2）；capability_id 必须落在 caller package namespace；audit redaction 同等
- **连接 ID == 流 ID**：`open` 返回的 `connection_id` 也是 `stream_id`，`kernel.capability.cancel(stream_id)` 可优雅关闭
- **专用事件信道**：WS 帧是混合 text/binary，与 SSE chunk 形状不同，使用 `kernel/outbound.websocket.*` 独立事件，避免污染 stream chunk 形状
- **send 为 unary 调用**：避免子进程 SDK 端复杂的双向流抽象；多次 send 是多次 RPC，背压由 executor 缓冲限额表达
- **默认 deny-all**：Profile 必须显式 `outbound.websocket.enabled: true` + executor 选择，否则全部拒绝

## 阶段

### Z0: 计划 push（本文件 + 英文版）

### Z1: 协议方法注册

新增三个公开协议方法：

| 方法 | 形式 | 说明 |
|------|------|------|
| `kernel.outbound.websocket.open` | streaming（订阅事件） | 建立 wss 连接，返回 connection_id |
| `kernel.outbound.websocket.send` | unary | 向已开连接发送一帧（text 或 binary） |
| `kernel.outbound.websocket.close` | unary | 主动发送关闭帧 |

`kernel.capability.cancel(connection_id)` 等价于 `close(1001, "canceled")`。

修改：
- `crates/ygg-runtime/src/protocol.rs` `KernelMethod` 枚举 + 字符串映射 + 注册表 + `streaming()` 标记
- `crates/ygg-runtime/src/runtime/protocol_dispatch.rs` 三个 dispatch 函数

### Z2: WebSocketExecutor trait + 三种实现

```rust
#[async_trait]
pub trait WebSocketExecutor: Send + Sync {
    async fn open(&self, req: OutboundWebSocketOpenRequest) -> Result<OutboundWebSocketSession>;
    async fn send(&self, connection_id: &str, frame: OutboundWebSocketFrame) -> Result<SendStatus>;
    async fn close(&self, connection_id: &str, code: u16, reason: Option<String>) -> Result<()>;
}

pub struct OutboundWebSocketSession {
    pub connection_id: String,
    pub subprotocol_negotiated: Option<String>,
    pub redaction_state: RedactionState,
    pub network_performed: bool,
    pub executor_kind: ExecutorKind,
}

pub enum OutboundWebSocketFrame {
    Text(String),
    Binary(Bytes),
}

pub enum SendStatus { Ok, BufferFull, ConnectionNotFound, ConnectionClosed }
```

实现：

| Executor | 行为 |
|----------|------|
| `DenyAllWebSocketExecutor` | 永远 `permission_denied` |
| `FakeWebSocketExecutor` | 内置可脚本化的本地 echo / 预编排帧序列，conformance 用 |
| `LiveWebSocketExecutor` | `tokio-tungstenite` + `rustls`，wss-only，host equality |

`LiveWebSocketExecutor` 必须强制：

- `wss://` 协议（除非 `allow_insecure_ws_for_tests` + 仅 loopback）
- `destination_host` 严格匹配握手 URL host（不允许 redirect — WS 本身没 redirect 概念，但要拒绝 `Sec-WebSocket-Location` 等异常头）
- secret_headers 在 handshake HTTP upgrade 阶段注入
- `max_frame_bytes` / `max_total_bytes_inbound` / `max_total_bytes_outbound` 限额
- `max_idle_ms`（无帧最大闲置）
- `max_duration_ms`（连接总时长上限）
- ping/pong 自动处理（不暴露给上层）
- 关闭码规范化（4000-4999 自定义码段允许，其他映射到标准码）

每个连接维护一个 actor（`tokio::spawn`）：
- 接收 send 命令通过 `mpsc::Sender<Command>`
- 收到对端帧 → 发出 `kernel/outbound.websocket.frame` 事件
- 关闭/错误 → 发出对应事件 + 清理状态

### Z3: Manifest schema 扩展

`network.declarations` 中 `methods` 数组新增字符串值 `"WEBSOCKET"`：

```yaml
permissions:
  network:
    declarations:
      - host: api.openai.com
        methods: [POST, WEBSOCKET]
```

dispatch 时：
- HTTP 类方法（POST/GET/...）→ outbound.execute / outbound.stream 路径
- `WEBSOCKET` → outbound.websocket.* 路径
- 不匹配则 fail-closed

修改：
- `crates/ygg-core/src/manifest.rs` validation：`WEBSOCKET` 是合法 method 名
- `crates/ygg-runtime/src/runtime/network.rs` 匹配逻辑增加 method 类型枚举

### Z4: HostProfile.outbound.websocket section

```yaml
outbound:
  execute: ...    # Y1
  websocket:      # 新增
    enabled: false                       # 默认禁用
    executor: deny_all                   # deny_all | fake | live
    allowed_hosts: []
    wss_only: true
    max_idle_ms: 60000
    max_duration_ms: 1800000             # 30 分钟
    max_frame_bytes: 65536               # 64 KB
    max_total_bytes_inbound: 10485760    # 10 MB
    max_total_bytes_outbound: 10485760
    max_concurrent_connections: 8
    allow_insecure_ws_for_tests: false
```

修改：
- `crates/ygg-cli/src/cli.rs` `HostProfileOutbound` 增加 `websocket` 字段
- `crates/ygg-cli/src/commands/host.rs` 把 profile 转成 `WebSocketExecutorConfig`，注入 Runtime

### Z5: 子进程 SDK kernelClient.openWebSocket

```typescript
export interface KernelWebSocketHandle {
  readonly connectionId: string;
  readonly subprotocol?: string;
  send(frame: { kind: 'text'; data: string } | { kind: 'binary'; data: Uint8Array }): Promise<void>;
  close(code?: number, reason?: string): Promise<void>;
}

export interface KernelWebSocketCallbacks {
  onOpen?: () => void;
  onFrame: (frame: { kind: 'text'; data: string } | { kind: 'binary'; data: Uint8Array }) => void;
  onClose?: (info: { code: number; reason: string }) => void;
  onError?: (err: { code: string; message: string }) => void;
}

export interface KernelClient {
  // existing: sendKernelRequest, streamKernelRequest
  openWebSocket(
    params: WebSocketOpenParams,
    callbacks: KernelWebSocketCallbacks,
  ): Promise<KernelWebSocketHandle>;
}
```

内部实现：
1. SDK 把 open 请求作为 `kernel.outbound.websocket.open` 反向调用
2. 收到 connection_id 后，订阅 `kernel/outbound.websocket.*` 事件流（按 connection_id 过滤）
3. 帧事件 → `onFrame`；关闭事件 → `onClose`；错误事件 → `onError`
4. `handle.send(frame)` 内部 → `kernel.outbound.websocket.send` 反向调用
5. `handle.close()` 内部 → `kernel.outbound.websocket.close` 或 `kernel.capability.cancel`

修改：
- `sdk/typescript/subprocess/index.ts` 增加 `openWebSocket` 与相关类型
- `sdk/typescript/subprocess/test/` 新增 ws mock 测试

### Z6: 出站完成审计事件（小缺口补丁）

新增三个完成事件，填补 Y1/Y3 在审计层的"开始有记录、结束无汇总"的缺口：

| 事件 | 何时触发 | 关键字段 |
|------|----------|----------|
| `kernel/outbound.execute.completed` | unary 出站终结 | status / executor_kind / total_bytes / duration_ms / network_performed / redaction_state |
| `kernel/outbound.stream.completed` | stream.ended/error/cancelled/timeout 后 | status / total_chunks / total_bytes / duration_ms / final_termination |
| `kernel/outbound.websocket.completed` | ws close 收发完成 | code / reason / total_frames_in/out / total_bytes_in/out / duration_ms |

WS 期间的事件：

| 事件 | 触发时机 | payload（脱敏） |
|------|----------|----------------|
| `kernel/outbound.websocket.opened` | handshake 成功 | connection_id / destination_host / capability_id / package_id / subprotocol |
| `kernel/outbound.websocket.frame` | 收/发一帧 | connection_id / direction / frame_kind / bytes / seq；不含 payload 内容 |
| `kernel/outbound.websocket.error` | 任意错误 | connection_id / error_code / message_redacted |

修改：
- `crates/ygg-core/src/event.rs` 新增事件类型
- `crates/ygg-runtime/src/runtime/network.rs` audit 写入路径增加 completed 事件发射

### Z7: Conformance

新增 case 类（约 12 条）：

```text
outbound_websocket_default_deny_all
outbound_websocket_fake_executor_open_send_close
outbound_websocket_secret_ref_undeclared_fails
outbound_websocket_capability_namespace_enforced
outbound_websocket_wss_only_default
outbound_websocket_idle_timeout_emits_error_and_completed
outbound_websocket_max_total_bytes_inbound_terminates
outbound_websocket_max_concurrent_connections_enforced
outbound_websocket_cancel_via_capability_cancel
outbound_execute_completed_audit_emitted
outbound_stream_completed_audit_emitted
outbound_websocket_completed_audit_emitted
```

预期总 case 数：347 → ~359

### Z8: Profile + 文档收敛 + 删除临时 plan

- 更新 `profiles/forge-with-live-models.example.yaml` 加入 `outbound.websocket` 注释段（默认 disabled，示意 OpenAI Realtime / Gemini Live host）
- 更新 `docs/protocol/PROTOCOL_V0.md` / `.en.md` outbound 章节加入 websocket 三方法 + completed 事件
- 更新 `docs/spec/CONFORMANCE_MATRIX.md` / `.en.md` 增加 websocket 行
- 更新 `README.md` / `README.en.md` outbound 描述为三件套
- 更新 `docs/ALPHA_STATUS.md` / `.en.md` 出站章节
- 更新 `docs/roadmap/NEXT_STEPS.md` / `.en.md`
- 删除 `docs/YGGDRASIL_OUTBOUND_WEBSOCKET_PLAN.md` 与 `.en.md`

## 安全清单（Z 阶段贯穿）

- [ ] 默认 deny_all
- [ ] 仅 wss:// 默认放行
- [ ] handshake host 严格匹配 destination_host
- [ ] secret_ref 必须 manifest 声明（Y2 规则）
- [ ] capability_id 必须属于 caller package namespace
- [ ] 帧 payload 永不进 audit；仅 size/kind/seq
- [ ] secret 永不进 frame events / completed events
- [ ] idle/duration/total-bytes/concurrent 四道限额
- [ ] capability.cancel 等价于 close(1001)
- [ ] 真实 wss smoke 仅在 `YGG_LIVE_WEBSOCKET_TESTS=1` + 凭据具备 时跑

## 完成判据

- `cargo test --workspace` 通过
- `cargo run -p ygg-cli -- conformance` 347 → ~359
- `forge-alpha.yaml` 仍可解析
- `forge-with-live-models.example.yaml` 加 websocket section 后仍可解析
- 真实 wss smoke 仅 opt-in；默认 CI 不联网
- 三完成事件在 fake/live 路径都触发并被 conformance 验证
- 删除临时 plan 文件，长期文档同步
