# 架构

> [English](./README.en.md) · [中文](./README.md)

内核、能力包与项目三层架构、能力包契约、扩展点、事件模型与生命周期。

## 平台立场

- [`VISION.md`](VISION.md) — 平台为何而存在
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — kernel + packages + projects 三层架构
- [`PLATFORM_KERNEL.md`](PLATFORM_KERNEL.md) — 内核做什么、不做什么
- [`CONSTITUTION_V2.md`](CONSTITUTION_V2.md) — 候选 v2 宪法：长期分层、不变量与反僵化约束；尚未替代现行契约

## 能力包契约

- [`CAPABILITY_PACKAGE.md`](CAPABILITY_PACKAGE.md) — 能力包契约
- [`EXTENSION_POINTS.md`](EXTENSION_POINTS.md) — 扩展点 / hook 契约
- [`EVENT_MODEL.md`](EVENT_MODEL.md) — 不透明事件日志模型
- [`RUNTIME_LIFECYCLE.md`](RUNTIME_LIFECYCLE.md) — 内核侧生命周期

## 上游集成边界

- [`PI_INTEGRATION.md`](PI_INTEGRATION.md) — pi agent 框架的吸收边界

## Host 控制平面

- [`HOST_DEVELOPMENT_CONTROL_PLANE.md`](HOST_DEVELOPMENT_CONTROL_PLANE.md) — 受控源码变更、验证、promotion 与恢复
- [`HOST_REMOTE_ACCESS.md`](HOST_REMOTE_ACCESS.md) — root / 设备身份、scope、HTTPS pairing 与显式应用路由暴露
- [`HOST_PROJECT_AUTHORITY.md`](HOST_PROJECT_AUTHORITY.md) — 项目级资源权威、认证上下文、session binding 与授权审计
- [`DURABLE_DEPLOYMENT_CONTROLLER.md`](DURABLE_DEPLOYMENT_CONTROLLER.md) — desired/observed state、幂等 operation、安全激活与恢复
- [`TARGET_AGENT_PROTOCOL.md`](TARGET_AGENT_PROTOCOL.md) — remote target 身份、类型化操作、artifact/secret 与 tunnel 边界
- [`OPERATIONS_DATA_RELEASE.md`](OPERATIONS_DATA_RELEASE.md) — 数据迁移、备份、健康、诊断、升级和供应链门禁
