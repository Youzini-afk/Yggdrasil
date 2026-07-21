# v1 版本策略

## 方法命名空间

所有 v1 内核方法都使用 `kernel.v1.*`。`kernel.v1.*` 与未来的 `kernel.v2.*` 是并行、互不兼容的命名空间；不会在 v1 方法名下引入破坏性语义变更，也不会为旧的 `kernel.*` 名称保留兼容层。

## Schema 规则

`docs/spec/v1/schemas/` 是 v1 的公开规范产物。v1 只能 additive-only 演进：

- 可以新增可选字段、事件类型、方法或 enum 变体（如果旧实现可安全忽略）。
- 不得删除字段、改变字段类型、把可选字段改为必填、收紧已发布 enum、改变事件 payload 的既有含义。
- schema 变更必须通过 `scripts/validate-schemas.sh`；CI 会设置 `BASE_SCHEMA_DIR`，检测文件删除以及常见结构性破坏，包括 type/const 变化、新增 required 字段、enum 收窄、property/definition 删除、边界收紧和不兼容的组合器变化。

## 何时进入 v2

当需要破坏性变更时进入 `kernel.v2.*`，例如：必填字段变化、已有字段类型变化、错误码语义变化、权限模型不兼容、或事件 payload 需要不可兼容重塑。v2 不会覆盖 v1；host 可同时暴露多个版本。

## 协商

包通过 `kernel.v1.host.info` 读取 `protocol_version`、方法列表、状态和传输列表。如果包需要 v1 方法，应检查该方法是否存在且状态不是 `planned`；如果未来需要 v2，应调用 `kernel.v2.host.info` 或读取 host 明确发布的 v2 方法列表。
