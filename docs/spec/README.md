# 规范

> [English](./README.en.md) · [中文](./README.md)

可执行 v1 契约和 hostile conformance 路线图。规范文件以代码 + 用例支撑——不是愿景文档。

- [`KERNEL_V1_CONTRACT.md`](KERNEL_V1_CONTRACT.md) — kernel v1 公开契约：57 个方法、41 个事件、能力句柄、Path A / Path B、SDK 与 conformance
- [`CONFORMANCE_MATRIX.md`](CONFORMANCE_MATRIX.md) — hostile conformance 用例清单（按 tag 与领域索引）
- [`v1/EVENT_KIND_REGISTRY.md`](v1/EVENT_KIND_REGISTRY.md) — v1 事件类型 registry
- [`v1/ERROR_CODES.md`](v1/ERROR_CODES.md) — v1 错误码
- [`v1/VERSIONING.md`](v1/VERSIONING.md) — v1 additive-only 版本策略
- [`v1/schemas/`](v1/schemas/) — 105 个 JSON Schema（methods / events / top-level），SDK 的单一可信源

跑全套 conformance：

```bash
cargo run -p ygg-cli -- conformance
```
