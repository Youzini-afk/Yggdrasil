# 规范

> [English](./README.en.md) · [中文](./README.md)

可执行 alpha 契约和 hostile conformance 路线图。规范文件以代码 + 用例支撑——不是愿景文档。

- [`KERNEL_V0_ALPHA_CONTRACT.md`](KERNEL_V0_ALPHA_CONTRACT.md) — kernel v0 alpha 契约矩阵：协议方法、状态、streaming 标志、命名空间、red lines
- [`CONFORMANCE_MATRIX.md`](CONFORMANCE_MATRIX.md) — hostile conformance 用例清单（按 tag 与领域索引）

跑全套 conformance：

```bash
cargo run -p ygg-cli -- conformance
```
