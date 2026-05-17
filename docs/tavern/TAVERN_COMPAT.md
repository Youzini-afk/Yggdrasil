# Tavern 兼容性（延后）

> [English](./TAVERN_COMPAT.en.md) · [中文](./TAVERN_COMPAT.md)

本文档为未来的能力包家族预留，该家族将导入 SillyTavern 资源并复现足够的 Tavern 行为让社区内容能在 Yggdrasil 上运行。它不在近期路径上。

## 立场

Tavern 兼容性不属于内核。内核对角色卡、世界书、预设、提示词渲染或任何其他内容形态的东西不携带任何意见。

当 Tavern 兼容性被构建时，它将以一个或多个能力包的形式交付，受同一套 manifest、fabric、权限和 sandbox 规则约束，与任何第三方包无异。它不会获得内核特权。

## 可能的形态（仅作草稿）

未来的 Tavern 包家族可能包含，作为独立包：

- 一个资源导入器，解析 Character Card V2、PNG 内嵌元数据、世界书、预设和聊天历史。
- 一个原生 projection 包，将这些转换为包定义的 asset 和事件。
- 一个行为层，复现 Tavern 式的提示词渲染和 lorebook 激活，供官方对话 runtime 包或 Tavern 形态的 runtime 包使用。
- 一个扩展 shim（适用时），将 Tavern 扩展概念映射到 Yggdrasil 能力。

内核看到的仅仅是：在自己 namespace 中声明事件类型、能力和 asset 的包，与任何其他包没有区别。

## 无损导入原则（承前）

当这项工作展开时，导入的资源保留其原始 payload，附带任何原生 projection。旧 schema 不应定义平台能表达什么，但也不应在导入时被销毁。

```text
original_payload   the original SillyTavern data, untouched
native_projection  package-defined views derived from it
```

这一原则属于导入器包，不属于内核。

## 内核的非目标

内核永远不会：

- 交付一个 SillyTavern 解析器，
- 建模角色卡或世界书，
- 硬编码 `{{char}}` / `{{user}}` 替换，
- 提供 Tavern 专用的 hook 或方法，
- 区别对待 Tavern 包和其他任何包。

## 状态

Tavern 兼容性延后至 Yggdrasil 上至少存在一个可玩的对话/runtime 能力包来消费 Tavern 形态的内容。它所需的平台底座 —— 包、事件、能力、hook、权限、surface contributions、proposal、asset、branch、projection —— 现已就位，因此 Tavern 兼容性在构建时可以完全以包的形式运行，无需内核变更。在此之前，本文档仅固定立场：Tavern 兼容性是未来的包家族，不是平台层。
