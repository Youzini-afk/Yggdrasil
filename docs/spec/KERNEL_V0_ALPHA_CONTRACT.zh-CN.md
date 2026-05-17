# 内核 v0 Alpha 契约

> [English](./KERNEL_V0_ALPHA_CONTRACT.md) · [中文](./KERNEL_V0_ALPHA_CONTRACT.zh-CN.md)

本文档是当前 Yggdrasil 内核 alpha 的实现契约。它有意比长期架构文档更窄：如果本矩阵说某行为是 `implemented`，代码和 conformance 必须证明它；如果说 `partial`，则类型或 API 存在但行为不完整；如果说 `planned` 或 `deferred`，则任何调用者不得依赖它。

关于当前运行内容的可执行快照，见 `docs/ALPHA_STATUS.md`。关于后续阶段，见 `docs/roadmap/NEXT_STEPS.md`。

Alpha 的目标不是可游玩体验。目标是可证伪的、内容无关的内核，使得包、capability、事件、权限和协议可以在没有特权官方路径的情况下被测试。Play/Forge Surface Contract Beta 建立在本契约之上；它不会放宽本契约。

## 契约状态语言

- `implemented`：已存在于代码中，并有测试或 CLI conformance 覆盖。
- `partial`：类型或 API 存在，但行为不完整或 conformance 不充分。
- `planned`：在契约中保留，尚未实现。
- `deferred`：已记录的目标，不在当前里程碑内。

## 内核对象契约

| 对象 | Alpha 状态 | 契约 |
|---|---:|---|
| `KernelSession` | implemented | 持有身份、标签、活跃包集、principal 范围、状态、时间戳、metadata。它不持有消息、回合、提示词、角色、世界或记忆。 |
| `EventEnvelope` | implemented | 只追加的不透明 JSON payload，具有每 session 的序列号、writer package id、带 namespace 的 kind、schema 版本、时间戳、metadata。 |
| `PackageManifest` | implemented | 声明身份、entry 形式、提供的 capability、消费的 capability、贡献的 schema/hook/extension point/asset、权限、sandbox 策略。 |
| `PackageRecord` | partial | 追踪 package id、版本、entry kind、计数、manifest、trust level、状态时间戳。Lifecycle 验证并注册 manifest 声明；`rust_inproc` entry 在提供的 capability 可以加载之前通过 host 目录解析；subprocess entry 在就绪之前启动 JSON-RPC stdio 进程并握手。对已实现的 entry 形式发出 loading/starting/ready/stopping/stopped/unloaded/degraded 事件。WASM/remote 仍为下一步。 |
| `CapabilityDescriptor` | implemented | 声明 provider 拥有的 capability id、版本、input/output schema 引用、streaming、副作用、描述。 |
| `HookSubscription` | partial | Manifest 声明的订阅已存在；hook 分发现已针对事件追加和 capability 调用 lifecycle 点运行，具备稳定排序、遗留 fixture handler、包拥有的 handler capability、metadata 变更和卸载清理。丰富的超时/错误审计仍为下一步。 |
| `AssetRecord` | partial | 不透明 asset put/get/list 已存在，包含 id、origin package、mime、hash、size、metadata 和 `kernel/asset.put` 审计事件。Asset 状态可以从持久事件日志 rehydrate；二进制/blob 存储和权限执行仍为下一步。 |

## 协议方法矩阵

| 方法 | 状态 | 备注 |
|---|---:|---|
| `kernel.session.open` | implemented | 开启内容无关 session 并写入 `kernel/session.opened`。 |
| `kernel.session.close` | implemented | 关闭 session 并写入 `kernel/session.closed`。 |
| `kernel.session.fork` | partial | 从父序列创建子 session 并记录 branch 族系，不解释内容。 |
| `kernel.session.branch.list` | partial | 列出与 session 相关的内存 branch 记录。 |
| `kernel.session.get` | planned | 尚未在 service/CLI 中暴露。 |
| `kernel.session.list` | planned | 尚未在 service/CLI 中暴露。 |
| `kernel.event.append` | implemented | 对非内核 writer 强制执行 writer namespace 和 `events.append`。 |
| `kernel.event.list` | implemented | 按 session 列出事件，支持 `after_sequence`、`limit`、`kind_prefix` 和 `writer_package_id`；运行时具备调用者感知的 `events.read` 门控，而 HTTP/CLI host 级别列表仍为 host-dev 本地管理。 |
| `kernel.event.subscribe` | partial | HTTP SSE 端点从 `after_sequence` replay 并 tail 实时事件。协议方法分发和 package-principal subscribe 权限仍为下一步。 |
| `kernel.package.load` | partial | 验证 manifest、host 策略，为 capability provider 解析 `rust_inproc` host entry，启动 subprocess JSON-RPC stdio entry，注册声明的 capability/hook，写入 lifecycle 事件。完整的过渡事件仍为 Platform Host Alpha 工作。 |
| `kernel.package.unload` | partial | 在存在 subprocess 句柄时停止，移除注册记录和声明的 capability/hook，写入 lifecycle 事件。 |
| `kernel.package.list` | implemented | 列出内存中的包记录。 |
| `kernel.package.status` | implemented | 返回 package id 的注册记录。 |
| `kernel.package.restart` | partial | 重启 subprocess entry 并发出 lifecycle 事件；其他 entry 形式被拒绝。 |
| `kernel.package.logs` | partial | 排出捕获的 subprocess stderr 日志并发出 `kernel/package.log` 事件；stdout 保留给 JSON-RPC 协议帧。 |
| `kernel.package.describe` | planned | 可以从 status manifest 派生，但尚未作为方法暴露。 |
| `kernel.capability.discover` | implemented | 列出已注册的描述符。 |
| `kernel.capability.describe` | planned | 注册表可以检查描述符；协议方法尚未暴露。 |
| `kernel.capability.invoke` | partial | 当提供调用者 package id 时强制执行调用者 capability 权限，除非提供 `provider_package_id` 否则检测模糊 provider，支持简单精确/主版本约束，根据支持的 schema 子集验证 capability input/output，通过 in-process package trait 执行 `rust_inproc` provider，并通过超时/降级处理执行 subprocess JSON-RPC stdio provider。 |
| `kernel.capability.stream` | partial | 描述符标志存在；stream start/cancel 生命周期在内存注册表中可用并带有序列事件。真实网络 streaming 延后。 |
| `kernel.capability.cancel` | partial | 内存 invocation registry 追踪进行中的 stream；cancel 标记 invocation 为 cancelled 并阻断后续 chunk。 |
| `kernel.extension_point.list` | implemented | 列出已注册的 extension point。 |
| `kernel.extension_point.describe` | planned | 注册表可以检查描述符；协议方法尚未暴露。 |
| `kernel.hook.list` | partial | 协议分发器可以列出已注册的 hook；公开文档和更丰富的过滤仍为 Platform Host Alpha 工作。 |
| `kernel.asset.put/get/list` | partial | 对 host-dev 协议调用者存在不透明 asset 底座，可以从 SQLite 支持的事件 rehydrate。Package-principal 权限检查和内容寻址 blob 存储仍为下一步。 |
| `kernel.projection.register/rebuild/get` | partial | 通用 projection 注册表存在，可以从 SQLite 支持的事件 rehydrate；rebuild 当前从过滤后的事件流计算事件计数/最后序列号。包拥有的 projection 执行仍为下一步。 |
| `kernel.host.info` | implemented | 返回协议版本、已公布的方法及其状态，以及当前支持的跨 in-process、HTTP `/rpc`、host stdio 和 ad hoc HTTP 的传输层标签。 |
| `kernel.host.ping` | partial | 已公布；直接 service 路由尚未暴露。 |
| `kernel.host.diagnostics` | partial | 返回包/capability/hook 计数和包记录，用于本地 host 可观测性。 |
| `kernel.host.principal` | planned | Identity provider 集成 deferred。 |
| `kernel.permission.grant/revoke/list/audit` | partial | Host-dev 调用者可以向 human 或 assistant principal 授予/撤销有范围权限、列出授权、检查授予/撤销审计事件。授权/撤销事件可从持久事件日志 rehydrate。完整资源策略覆盖仍为下一步。 |
| `kernel.proposal.create/get/list/approve/reject/apply` | partial | 用于需要审批的游创变更的通用 proposal lifecycle。初始 apply 支持覆盖 `asset.put` 和 `projection.rebuild`；proposal operations/expected effects 中的 raw secrets 会被拒绝；更广泛的事务和 revert/compensation 仍为下一步。 |
| `kernel.surface.contribution.list` | partial | 列出类型化的包声明 surface 描述符，涵盖 experience entry、Home/Play、Forge、asset 编辑器和 assistant slot。内核仅存储描述符；UI 渲染和内容语义仍为包/客户端工作。 |
| `kernel.surface.contribution.describe` | partial | 按 id 描述一个已声明的 surface contribution。 |

## 内核事件类型矩阵

| 事件类型 | Writer | 状态 | 触发 |
|---|---|---:|---|
| `kernel/session.opened` | kernel | implemented | Session 开启。 |
| `kernel/session.closed` | kernel | implemented | Session 关闭。 |
| `kernel/session.forked` | kernel | implemented | Session fork 创建 branch 族系。 |
| `kernel/package.loaded` | kernel | implemented | Manifest 被接受并注册。 |
| `kernel/package.loading` | kernel | implemented | 包记录进入 loading。 |
| `kernel/package.starting` | kernel | implemented | Subprocess 包进程即将启动/握手。 |
| `kernel/package.ready` | kernel | implemented | 包在 entry 特定启动后就绪。 |
| `kernel/package.stopping` | kernel | implemented | Unload/restart 正在停止包执行。 |
| `kernel/package.stopped` | kernel | implemented | 包执行已停止。 |
| `kernel/package.unloaded` | kernel | implemented | 包从注册表移除。 |
| `kernel/package.degraded` | kernel | implemented | 实际包执行失败/健康丢失。 |
| `kernel/package.log` | kernel | implemented | 捕获的 subprocess stderr 日志行。 |
| `kernel/asset.put` | kernel | implemented | 不透明 asset 已存储。 |
| `kernel/projection.updated` | kernel | implemented | 通用 projection 状态已 rebuild。 |
| `kernel/capability.invoked` | kernel | planned | 调用 lifecycle 事件。 |
| `kernel/capability.completed` | kernel | planned | 调用成功事件。 |
| `kernel/capability.failed` | kernel | planned | 调用失败事件。 |
| `kernel/stream.started` | kernel | partial | Streaming invocation 已启动。 |
| `kernel/stream.chunk` | kernel | partial | Streaming chunk frame 已发出。 |
| `kernel/stream.progress` | kernel | partial | Streaming 进度指示。 |
| `kernel/stream.ended` | kernel | partial | Streaming invocation 正常结束。 |
| `kernel/stream.error` | kernel | partial | Streaming invocation 发生错误。 |
| `kernel/stream.cancelled` | kernel | partial | Streaming invocation 被调用者取消。 |
| `kernel/stream.timeout` | kernel | partial | Streaming invocation 超时。 |
| `kernel/permission.denied` | kernel | implemented | 权限拒绝审计。 |
| `kernel/permission.granted` | kernel | implemented | 权限授予审计。 |
| `kernel/permission.revoked` | kernel | implemented | 权限撤销审计。 |
| `kernel/proposal.*` | kernel | partial | Proposal lifecycle 审计事件。 |
| `kernel/outbound.request` | kernel | partial | 出站网络请求被允许并审计。 |
| `kernel/outbound.denied` | kernel | partial | 出站网络请求被拒绝。 |
| `kernel/error` | kernel | planned | 通用结构化内核错误事件。 |

非内核事件类型必须以 writer package id 加 `/` 开头。内核必须拒绝包尝试写入 `kernel/...` 或其他包的 namespace。

## 包入口矩阵

| Entry 形式 | Manifest 状态 | 执行状态 | Trust level |
|---|---:|---:|---|
| `rust_inproc` | implemented | partial | `trusted_inproc` |
| `subprocess` | implemented | partial | `process_isolated` |
| `wasm` | implemented | deferred | `wasm_sandbox` |
| `remote` | implemented | deferred | `remote_boundary` |

Manifest 支持意味着 schema 可以描述 entry 且 host 策略可以接受/拒绝它。执行支持意味着内核实际跨该边界调用。`rust_inproc` 现在通过 host 提供的 package trait 和目录执行。Subprocess JSON-RPC stdio 执行现在支持握手/调用/超时/卸载 kill；更完整的 lifecycle 事件排序仍为 Platform Host Alpha 工作。WASM 和 remote 执行仍为 deferred。

## 权限矩阵

| 权限 | 状态 | 当前执行 |
|---|---:|---|
| `events.append` | implemented | 非 kernel `event.append` 所需。 |
| `events.read` | partial | 运行时支持包 manifest 检查和 human/assistant principal 的有范围授权。SSE subscribe 当前仅限 host-dev。 |
| `capabilities.invoke` | partial | 运行时支持包 manifest 检查和 human/assistant principal 的有范围授权。匿名 host 调用仅作为 host/dev 操作被允许，且绝不能成为包特权。 |
| `packages.call` | planned | 包到包控制平面未实现。 |
| `assets.read/write` | planned | Asset 存储未实现。 |
| `projections` | planned | Projection 注册仅限 host-dev；包权限模型仍为下一步。 |
| `network.hosts` | partial | 包在 manifest 中声明允许的出站 host；runtime `check_network_policy` 和 `check_and_audit_outbound` 为 Ygg 提供的 network helper 强制执行 allowlist。支持扁平 `hosts` 列表和结构化 `declarations`（host、methods、purpose）。官方包无绕过。被拒绝的请求写入 `kernel/outbound.denied`；被允许的请求写入 `kernel/outbound.request` 并带 redacted audit。 |
| `filesystem.read/write` | planned | 当 subprocess/WASM 执行存在时适用。 |

## Secret reference contract（Phase S1）

| 项目 | 状态 | 当前执行 |
|---|---:|---|
| `SecretRef` 类型与校验 | implemented | 识别 `secret_ref:<vault>:<key>`、`secretRef:`、`secret-ref:` 和 `host:` patterns。 |
| Host secret resolver | partial | `HostSecretResolver` trait 和 deny-all resolver placeholder 存在；生产级 vault/secret store 属于 host-level 后续工作。 |
| Proposal 中的 raw-secret blocking | implemented | operation payloads 或 expected_effects 中有 raw secrets 的 proposal 会被拒绝；`secret_ref` references 会被接受。 |
| Asset metadata 中的 raw-secret blocking | implemented | 有 raw secrets 的 asset metadata 会被拒绝。Asset content 不扫描（任意用户数据）。`secret_ref` references 会被接受。 |
| Official bypass | implemented | Official-looking packages 不会绕过 secret scanning 或 permission rehydrate。 |

## Lifecycle 规则

已实现：

1. Session open/close 写入内核事件。
2. 包加载验证 manifest 和 host 策略，注册 manifest 声明的 capability/hook/extension point，写入内核事件。
3. 包卸载移除注册声明并写入 stopping/stopped/unloaded 内核事件。
4. 事件追加分配序列号/时间戳/id 并强制执行 namespace 所有权。
5. 权限拒绝写入 `kernel/permission.denied` 审计事件。
6. 已关闭 session 拒绝非内核追加。
7. Capability input/output 和包声明的事件 payload schema 根据当前 JSON Schema 子集验证。
8. 协议上下文区分 host/dev 调用和 package-principal 调用，package-principal 操作忽略调用者提供的包身份字段。
9. 规范协议信封可以 in-process 和通过 HTTP `/rpc` 分发；`ygg host-stdio` 通过 stdin/stdout 暴露相同信封用于自动化。
10. Subprocess JSON-RPC stdio 包可以握手、调用 capability、超时、降级、重启、捕获 stderr 日志和带进程 kill 的卸载。
11. 第一个 hook fabric 切片针对事件/capability before/after 点分发，具备稳定排序、遗留 veto fixture、包拥有的 handler capability、metadata 变更和卸载清理。
12. 事件范围 replay 已为 in-process 协议和 HTTP ad hoc 列表实现；HTTP SSE 可以从 `after_sequence` replay 并 tail 新事件。
13. Capability 路由支持显式 provider 选择和简单精确/主版本约束。
14. Asset、branch 和通用 projection 底座对 host-dev 协议调用者存在，可以从持久事件日志 rehydrate。
15. Human 和 assistant principal 可以接收事件读取和 capability 调用的有范围授权，附带授予/撤销审计事件。
16. 第一批官方基础包（`official/package-lab`、`official/schema-tools`、`official/event-tools`）通过普通 manifest 加载并通过普通 capability/surface 描述符路由。
17. `official/assistant-lab` 是一个普通 assistant capability 包，它返回需要审批的 proposal 而非直接修改受信状态。
18. 第一个空白游创循环 demo 证明包启动、assistant proposal、branch fork、asset 写入和 projection rebuild 无需向内核添加内容语义。
19. 通用 proposal lifecycle 方法将 assistant/包变更置于显式审批之后，并追加审计事件。
20. Permission grant/revoke events 可从持久事件日志 rehydrate。重启同一 SQLite-backed runtime 后，human/assistant principal 的作用域授权仍生效。
21. Secret references 遵循 `secret_ref:<vault>:<key>` contract。Proposal payloads 和 asset metadata 中的 raw secrets 会被 kernel 拒绝。Content/description/title/reason fields 不做 value-pattern 扫描，以避免误伤普通文本。
22. Host secret resolution 只通过 `HostSecretResolver` 边界表达。解析后的 raw secret 不得被写回 events、proposals、logs 或 audit records。
23. 网络权限声明：包在 `permissions.network` 中声明允许的出站目的地（结构化 `declarations` 含 host、methods、purpose；或扁平 `hosts` 向后兼容）。Runtime 策略检查器为 Ygg 提供的 network helper 强制执行 allowlist。官方包无绕过。
24. Outbound audit records：`OutboundAuditRecord` 捕获 principal、package_id、capability_id、destination_host、method、purpose、redaction_state、secret_refs_used、usage/cost 占位符和 status/error。Raw body/header/prompt/response 不会被保存。`redaction_state` 默认为 `redacted`。
25. 被拒绝的出站请求写入 `kernel/outbound.denied` 事件；被允许的请求写入 `kernel/outbound.request` 事件。两者可通过 `kernel.outbound.audit` 检查。
26. Streaming invocation registry：`StreamRegistry` 追踪进行中的 streaming capability 调用，支持 start/append/end/cancel/timeout 生命周期。`StreamFrameEnvelope` 定义通用内容无关的 frame 类型（start/chunk/progress/end/error/cancelled/timeout），包含 invocation_id、stream_id、sequence、redaction_state 和 timestamp/metadata。不包含 model/prompt/agent 语义。
27. `kernel.capability.stream` 在启动 streaming invocation 前验证 descriptor 中 `streaming=true`；非 streaming 能力（descriptor `streaming=false`）被拒绝。
28. Cancel 将活跃的 streaming invocation 标记为 `Cancelled` 并阻断后续 chunk/progress frame。Timeout 将 invocation 标记为 `Timeout` 并阻断后续 frame。Error 终端 frame 将状态设为 `Error` 并阻断后续 frame。正常 end 将状态设为 `Ended`。
29. Streaming 生命周期按序发出 kernel 事件：`kernel/stream.started`（启动）、`kernel/stream.chunk`（chunk）、`kernel/stream.progress`（进度）、`kernel/stream.ended`（正常结束）、`kernel/stream.error`（错误）、`kernel/stream.cancelled`（取消）、`kernel/stream.timeout`（超时）。
30. `StreamInvocationRecord` 追踪 invocation_id、stream_id、capability_id、provider_package_id、session_id、状态、frame_count、时间戳和 metadata。终态阻断后续 frame 追加。

Platform Host Alpha 仍为 partial 的项目：

1. 事件 subscribe 缺少协议分发的 streaming 和 package-principal subscribe 权限。
2. Hook handler 超时/错误审计不充分。
3. 包 lifecycle 为已实现的 entry 形式发出过渡事件；lifecycle 健康检查和更丰富的崩溃监控仍为 partial。
4. Capability 路由有简单的显式 provider/版本约束，但没有持久的 provider 选择策略。
5. 传输层 conformance 覆盖核心 `/rpc` 和 host stdio 行为，但不是完整的方法一致性矩阵。
6. Asset/projection/branch 底座通过事件日志持久化，但尚未执行 package-principal 权限或使用专用 blob 存储。
7. 生产级 secret vault 集成延后为 host-level 包；`DenyAllSecretResolver` 是默认值。
8. 网络权限强制执行覆盖 Ygg 提供的 network/request helper；不声称拦截任意 subprocess OS 级别的出站请求。

下一步：

1. 包 lifecycle 必须运行实际的 entry 握手/注册/启动/停止。
2. 包加载应暴露显式的 discovered/loading/starting/ready 过渡，而非直接的 ready 记录。
3. Capability lifecycle 必须写入 invoked/completed/failed 事件。
4. 内核操作必须根据 extension-point 契约分发 before/after hook；事件追加和 capability 调用拥有第一个可执行切片。
5. Session 包集必须约束路由。
6. Schema 验证必须从当前实用子集发展到已发布的完整 schema 方言。

## Schema 验证子集

Alpha 验证一个刻意保持小巧的 JSON Schema 兼容子集：

- `null` 或 `{}` 表示接受任何 JSON 值。
- `type` 可以是 `object`、`array`、`string`、`number`、`integer`、`boolean` 或 `null`。
- 对 object 字段强制执行 `required`。

这足以使 schema 声明在 conformance 中可执行，而不会过早冻结完整的 schema 方言。

## 内容无关不变量

内核 crate 不得定义或要求内容形态的概念，如 `Turn`、`Message`、`PromptFrame`、`ModelCall`、`Agent`、`World`、`Scene`、`Director` 或 `Memory`。任何此类概念都属于某个包。
