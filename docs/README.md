# Yggdrasil 文档

> [English](./README.en.md) · [中文](./README.md)

按主题分组的开发文档导航。每篇都同时提供英文与简体中文版本，文件顶部的双语导航 blockquote 可在两种语言间切换。

## 立场与现状

- [`CHARTER.md`](CHARTER.md) — 不变的根本原则
- [`ALPHA_STATUS.md`](ALPHA_STATUS.md) — 已完成 / 部分完成 / 延后内容的活快照
- [`../BUILDING.md`](../BUILDING.md) — Rust、Web、Tauri desktop 与 release 构建说明
- [`product/`](product/README.md) — 游创一体产品立场与体验牵引平台路线

## 架构与协议

- [`architecture/`](architecture/README.md) — kernel + packages 两层架构、能力包契约、扩展点、事件模型、生命周期
- [`protocol/`](protocol/README.md) — 公开协议规范
- [`spec/`](spec/README.md) — 可执行 alpha 契约矩阵、hostile conformance 路线图

## 创作

- [`guides/`](guides/README.md) — 能力包创作指南，按域分组（基础 / agent / 模型 / 推理 / 体验 / 记忆 / 存储 / 外部项目 / 分发）
- [`guides/SURFACE_HOSTING.md`](guides/SURFACE_HOSTING.md) — `clients/web` iframe SurfaceHost 与第三方 Web surface bundle 托管

## 性能与路线图

- [`performance/`](performance/README.md) — 性能基线、conformance 反馈环、代码健康
- [`roadmap/`](roadmap/README.md) — 当前与下一阶段、模型推理前置条件
- [`tavern/`](tavern/README.md) —— Yggdrasil 与 SillyTavern 兼容接入项目 YdlTavern 的关系

## 最短读路径

| 你想 | 先读 |
|---|---|
| 理解平台立场 | [`CHARTER.md`](CHARTER.md) → [`architecture/VISION.md`](architecture/VISION.md) → [`product/PLAY_CREATION_MODEL.md`](product/PLAY_CREATION_MODEL.md) |
| 理解架构 | [`architecture/ARCHITECTURE.md`](architecture/ARCHITECTURE.md) → [`architecture/PLATFORM_KERNEL.md`](architecture/PLATFORM_KERNEL.md) → [`architecture/CAPABILITY_PACKAGE.md`](architecture/CAPABILITY_PACKAGE.md) |
| 接入公开协议 | [`protocol/PROTOCOL_V0.md`](protocol/PROTOCOL_V0.md) → [`spec/KERNEL_V0_ALPHA_CONTRACT.md`](spec/KERNEL_V0_ALPHA_CONTRACT.md) |
| 写第一个能力包 | [`guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](guides/PACKAGE_AUTHORING_WALKTHROUGH.md) |
| 从 git 安装能力包 | [`guides/GIT_PACKAGE_INSTALLATION.md`](guides/GIT_PACKAGE_INSTALLATION.md) |
| 挂载第三方 Web surface | [`guides/SURFACE_HOSTING.md`](guides/SURFACE_HOSTING.md) |
| 构建 Web / Desktop / Release | [`../BUILDING.md`](../BUILDING.md) |
| 看当前状态 | [`ALPHA_STATUS.md`](ALPHA_STATUS.md) |
| 看下一步 | [`roadmap/NEXT_STEPS.md`](roadmap/NEXT_STEPS.md) |
