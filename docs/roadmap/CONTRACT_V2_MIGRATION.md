# Contract v2 分层迁移计划

> [English](./CONTRACT_V2_MIGRATION.en.md) · [中文](./CONTRACT_V2_MIGRATION.md)

> 这是临时实施计划。分层迁移完成后应删除本文，并把长期有效的结果收敛进
> architecture、spec、guides 与状态文档。

## 目标

在不推倒现有 runtime、Web、CLI、SDK 和 conformance 的前提下，把当前 `kernel.v1.*` 大合同迁移成职责明确、可协商、可弃用的分层合同：

- Constitutional Substrate；
- Host Control Plane；
- Protocol Commons；
- Shell / Product Profiles；
- Legacy Adapters。

目标边界见 [`CONSTITUTION_V2.md`](../architecture/CONSTITUTION_V2.md)，逐项归属见
[`CONTRACT_LAYERING_MATRIX.md`](../spec/CONTRACT_LAYERING_MATRIX.md)。

## 非目标

- 不一次性重写 runtime。
- 不立即删除任何 `kernel.v1.*` 方法或 schema。
- 不把 `World`、`Agent` 或 `Surface` 重新硬编码进 substrate。
- 不立即要求所有组件迁移到 WASM。
- 不在本计划中建设 marketplace、远程 registry 或经济系统。
- 不把设计文档本身当作 v2 已实现的证明。

## 迁移约束

1. 当前 Web 和 CLI 必须可以在兼容窗口内不修改继续运行。
2. 每次迁移先增加 canonical 路由和 adapter，再迁移调用者，最后才允许弃用旧名。
3. v1 原始 request、response、event 和未知字段必须可以无损保留。
4. 新合同默认 Experimental；未通过成熟度门槛不得宣称 Stable。
5. 安全与权限语义不得因 namespace 迁移而放宽。
6. 历史重放不得触发网络、模型、进程或其他外部效果。
7. 每项结构变化都需要 schema、SDK 和 conformance 同步。

## 当前实施状态

- [x] Phase 1：修复 v1 事实漂移、SDK/CI/Windows 基线。
- [x] Phase 2：建立 Experimental Contract Registry、集中 alias、显式 profile/version 协商与 SDK/conformance 生成链。
- [x] Phase 3：为 Host Control Plane、host bundle resolver、Shell contribution、Change/Proposal 与 Projection 建立 owner namespace 双栈；36 条 legacy alias 仍进入原 handler。
- [x] Phase 4：建立 SHA-256 ObjectStore、ArtifactDescriptor、asset adapter、旧 FNV 事件迁移与独立持久对象目录。
- [ ] Phase 5–9：receipt/change primitives、Protocol Commons、component identity、World Bundle 与客户端弃用迁移。

## 立即冻结线

在分层迁移完成前：

- 除安全修复、正确性修复和兼容字段外，不新增 `kernel.v1.*` 方法或事件；
- 新实验放在明确 owner 的实验 namespace，不扩大 v1 稳定承诺；
- `project`、`target`、`exec`、`port`、`proxy`、`surface slot` 不再被用作证明 substrate 应继续扩张的先例；
- 新领域语义先进入 package-owned experiment，经过协议化评审后才进入 Protocol Commons。

## 实施顺序

### 1. 先修复当前事实漂移

这是低风险前置工作，不改变合同语义。

交付：

- 将 `EVENT_KIND_REGISTRY.md` 补齐到 59 个事件，加入 `deployment.health`；
- 对齐 `KernelMethod::status()`、Contract 方法矩阵和 actual dispatch；
- 修正 Contract 中 capability/outbound namespace 数量和遗漏的 deployment hub 计数；
- 修复 Rust SDK crate root，使生成的 methods/events/types 真正导出；
- 把 Web 的 13 组测试加入 CI；
- 修复 Windows 下硬编码 `/tmp` 与 `python3` 的两项 runtime 测试。

验收：

```text
cargo check --workspace
cargo test -p ygg-core
cargo test -p ygg-runtime --lib
cargo test -p ygg-cli
npm run check --prefix clients/web
npm test --prefix clients/web
cargo run -p ygg-cli --bin validate-schemas
```

### 2. 建立合同 registry 与 alias 基础

在现有 `KernelMethod` 单一事实来源上增加显式元数据，不先移动 handler。

非规范性 registry 形状：

```text
ContractMethod
├── canonical_id
├── aliases[]
├── owner_layer
├── maturity
├── request_schema
├── response_schema
├── request_adapter
├── response_adapter
├── introduced_in
├── deprecated_in
└── replacement
```

交付：

- dispatcher 先解析 alias，再调用 canonical handler；
- alias 使用集中 registry，禁止散落字符串特判；
- `host.info` 返回 layers、versions、profiles、maturity 与 aliases；
- generated SDK 能生成 legacy alias 和 canonical client；
- conformance 新增 alias 等价、未知版本拒绝、禁止静默降级测试。

第一版 alias 可以是 identity adapter，证明路由机制而不改变 payload。

验收：

- 同一请求通过 legacy ID 和 canonical ID 得到语义等价结果；
- permission、principal、audit 和 error mapping 完全一致；
- 客户端要求不存在的 profile 时明确失败，不自动回落到更弱语义。

### 3. 只改 owner，不改行为

建立目标 namespace 的双栈入口，handler 仍调用现有实现。

首批迁移：

- `target/exec/port/proxy/project` → Host Control Plane；
- `surface.*` → host bundle resolver + `ygg.shell.default/v1` profile；
- `proposal.*` → Experimental Change protocol；
- `projection.*` → Experimental Projection protocol。

旧 `kernel.v1.*` 保留为 adapter。此步骤不引入新的 World、receipt 或 object store 数据结构。

验收：

- Web 继续只用旧 SDK 也能工作；
- 新 CLI smoke 可以只调用分层 namespace；
- legacy 与 canonical 路径共享同一 handler，不产生双实现漂移。

### 4. 建立内容寻址 object/artifact 基础

迁移前 `AssetRecord.hash` 使用 `fnv1a64:`，且 `asset.put` 把完整 content 写进 event metadata。该旧形状只适合确定性测试，不适合作为跨宿主、对抗碰撞的持久身份。v2 object identity 改用碰撞安全摘要，并让日志引用对象。

交付：

- `ArtifactDescriptor { artifact_type_uri, media_type, digest, size_bytes, references, annotations }`；
- `ObjectStore` trait：put、get、has、verify、stream；
- 初始必选 digest 为 `sha256:`，读取器保留算法前缀以允许未来扩展；
- object bytes 与 metadata 分离；
- journal/event/receipt 只保存 descriptor/ref，不复制大内容；
- `asset.put/get/list` adapter 映射到 object/artifact API；
- FNV 旧地址只作为 legacy alias，不能晋升为 v2 canonical identity。

数据迁移：

- 读取旧 asset 时计算 SHA-256 descriptor；
- 保存旧 asset id、旧 FNV hash 和原始 v1 event 引用；
- 迁移必须幂等，可中断后继续；
- 未知 artifact type 可以无损复制和导出。

验收：

- 相同 bytes 在两个 host 得到相同 digest；
- 篡改 bytes 必须验证失败；
- 不加载原 package 也能复制和检查对象；
- 大对象不再完整进入 event metadata。

实施结果（2026-07-21）：上述交付与验收已落地。公开细节见 [`OBJECT_STORE.md`](../spec/OBJECT_STORE.md)，可执行证据为 `asset.put_get_list`、`asset.legacy_fnv_migration`、`object_store.portability_integrity` 与 `substrate.sqlite_rehydrate`。

### 5. 引入 EffectReceipt 与 change primitives

先覆盖已经存在且边界清楚的 effect：capability invoke、outbound HTTP/stream/WebSocket 和 local exec。

交付：

- versioned `EffectReceipt` artifact/profile；
- `Intent`、`ChangeSet`、`PolicyDecision`、`Commit` 的 Experimental schema；
- capability/outbound/exec terminal path 生成 receipt；
- receipt 引用 input/output objects、component digest、authority、policy、approval、cost、latency、trace 与 parent receipts；
- Proposal adapter 将旧 lifecycle 转成 Change protocol；
- receipt 默认不包含 raw body、header、secret、完整 prompt 或完整用户内容。

历史重放：

- 读取 recorded output refs，不调用 executor；
- 缺失对象时明确报 incomplete history；
- 重新执行必须创建新 branch/head 和新 receipt，不能覆盖旧 receipt。

验收：

- 关闭所有 outbound executor 后，历史调用仍可重放；
- re-execute 产生不同 branch，旧历史不变；
- denied/cancelled/timeout/partial/success 均有可区分 terminal receipt；
- raw-secret 扫描覆盖 receipt 和 adapter 输出。

### 6. 建立 Protocol Commons 脚手架

交付 protocol descriptor，而不是先发明大量领域协议。

```text
ProtocolDescriptor
├── protocol_id
├── version
├── maturity
├── schemas / WIT worlds
├── semantic specification
├── lifecycle / state machine
├── error model
├── authority requirements
├── conformance vectors
├── compatibility profiles
├── migrations / adapters
└── conforming implementations
```

第一批只孵化三个协议：

- Change protocol：吸收现有 Proposal；
- Shell Default profile：吸收固定 SurfaceSlot；
- World Bundle Experimental profile：用于真实可移植性证明。

Projection 保持 Experimental，直到至少两个不同体验证明共享语义足够稳定。

验收：

- protocol conformance 与 implementation/package conformance 分开报告；
- 官方与第三方实现以相同向量测试；
- protocol major mismatch 有明确 adapter 或拒绝，不依赖 schema 恰好能解析。

### 7. 分离 package envelope 与 component identity

交付：

- package 继续负责获取、完整性和安装事务；
- package 内 artifact/component/protocol/content/surface 各有 descriptor 与 digest；
- component identity 不再等于 package id；
- composition lock 锁定 protocol profile、component digest 和 content roots；
- trust class 进入 component record 与 conformance report；
- `contract:none` 映射为 `foreign_capsule`，不颁发 conforming/portable claims。

执行边界：

- `trusted_native` 只用于显式宿主信任；
- `isolated_process` 不宣称 OS 网络/文件系统隔离，除非 host 有可验证强制；
- `sandboxed_component` 作为 AI 生成组件的首选候选，但在 WASI 0.3 工具链和 host 支持成熟前不强制；
- remote 实现必须显式身份、租户、网络和撤销语义。

验收：

- 同一 protocol implementation 可以在不同 package 中发布而保持 component identity/behavior claim；
- 替换 component 不改变 content roots；
- Foreign Capsule 可以启动，但 conformance 报告明确显示不具备可组合和可移植保证。

### 8. 用真实 World Bundle 证明边界

第一个压力源使用 `official/playable-creation-board`，不先造新的大型体验。

Experimental World Bundle 至少包含：

```text
WorldBundle
├── bundle_descriptor
├── world_head
├── journal_ranges
├── object_descriptors
├── composition_lock
├── protocol_profiles
├── policy_refs
├── effect_receipts
├── lineage
└── original_v1_envelopes
```

验收流程：

1. 在 host A 启动 playable board 并产生状态、分支和至少一次受控 effect。
2. 导出 bundle，验证所有 digest 和引用闭包。
3. 在全新 data dir 的 host B 导入。
4. 不加载原组件、不启用网络和模型，完成历史审计与确定性重放。
5. 安装不同实现后重新执行一个步骤，生成新 branch/head。
6. 用 headless CLI 读取同一世界，证明 Web shell 不是数据依赖。

失败条件：

- bundle 依赖 host A 的绝对路径；
- 缺少原 package 时历史无法读取；
- replay 触发真实外部调用；
- unknown artifact 被丢弃；
- import 后对象 digest、lineage 或 receipt 改变。

### 9. 迁移客户端并开始弃用

迁移顺序：

1. generated SDK；
2. CLI；
3. Web protocol client；
4. subprocess SDK；
5. 官方 packages；
6. 第三方示例与 guides。

每个 legacy method 只有满足以下条件才能进入 Deprecated：

- canonical replacement 已是 Candidate 或 Stable；
- legacy/canonical 等价 conformance 通过；
- SDK 和 migration tool 已发布；
- `host.info` 能报告 replacement；
- 支持窗口已写明；
- 至少一个真实项目已完成迁移。

进入 Legacy Adapter 后，旧方法只接受安全修复和数据读取兼容，不再增加新字段语义。

## Conformance 重组

保留现有具名 case/tag runner，增加以下套件：

| 套件 | 证明 |
|---|---|
| `substrate` | identity、authority、objects、journal、receipts、stream、transaction |
| `host` | project、exec、port、proxy、secret、deployment 的宿主行为 |
| `protocol:<id>` | 共同语义、状态机、错误和行为向量 |
| `shell:<profile>` | descriptor 映射、bridge authority、shell independence |
| `legacy` | alias 等价、无损转换、支持窗口与 deprecated diagnostics |
| `portability` | 跨 host bundle、offline replay、unknown artifact preservation |

Stable substrate 的发布门槛必须包含 portability 套件，而不只是单 host 单元测试。

## 迁移完成定义

完成不是「所有 `kernel.v1.*` 都被重命名」。完成需要同时满足：

- owner layering 在代码、schema、SDK 和文档中一致；
- legacy alias 有集中 registry 和 conformance；
- package 不再是唯一 artifact/component/content identity；
- object identity 使用可验证的碰撞安全摘要；
- effect receipt 覆盖非确定性和外部效果；
- Proposal 与固定 SurfaceSlot 已成为协议/profile；
- Foreign Capsule 不再宣称完整生态平权；
- World Bundle 通过跨 host、offline replay 和 shell independence 验收；
- 至少一个错误抽象完成 Deprecated → Legacy Adapter 的真实迁移。
