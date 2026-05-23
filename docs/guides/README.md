# 创作指南

> [English](./README.en.md) · [中文](./README.md)

按领域分组的能力包创作指南。每篇都对应一类能力包或一段创作循环，全部建立在公开协议、清单与 surface 契约之上。

## 起步

- [`PACKAGE_AUTHORING_WALKTHROUGH.md`](PACKAGE_AUTHORING_WALKTHROUGH.md) — 第三方能力包创作 walkthrough（init-package、check、run-fixture、reload、composition）
- [`CAPABILITY_HANDLES.md`](CAPABILITY_HANDLES.md) — 内核 v1 能力句柄模型、衰减、撤销、bindings 与 effect audit
- [`CONFORMANCE_KIT.md`](CONFORMANCE_KIT.md) — 第三方包本地验证 v1 contract compliance
- [`PATH_B_SELF_CONTAINED.md`](PATH_B_SELF_CONTAINED.md) — 自包含 Path B 包（`entry.contract: "none"`）
- [`GIT_PACKAGE_INSTALLATION.md`](GIT_PACKAGE_INSTALLATION.md) — 从公开 HTTPS git 仓库安装能力包（git outbound、profile lockfile、installer-lab）
- [`SURFACE_HOSTING.md`](SURFACE_HOSTING.md) — iframe SurfaceHost、第三方 Web surface bundle 约定与 host bridge

## 创作能力族

- [`CREATIVE_CAPABILITY_KIT.md`](CREATIVE_CAPABILITY_KIT.md) — Yggdrasil 通用创作能力包（persona / knowledge / context / text-transform）
- [`MODEL_CONNECTIVITY_KIT.md`](MODEL_CONNECTIVITY_KIT.md) — model provider profile 与 route planning kit
- [`MODEL_PROVIDER_INTEGRATION.md`](MODEL_PROVIDER_INTEGRATION.md) — 多 provider 模型接入（OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks）
- [`INFERENCE_CAPABILITY_AUTHORING.md`](INFERENCE_CAPABILITY_AUTHORING.md) — 与传输无关的推理能力包创作

## Agent 与体验

- [`AGENT_PACKAGE_AUTHORING.md`](AGENT_PACKAGE_AUTHORING.md) — 类 agent 能力包创作
- [`AGENTIC_FORGE_PACKAGE_AUTHORING.md`](AGENTIC_FORGE_PACKAGE_AUTHORING.md) — Agentic Forge runtime 能力包（计划图、scratch 分支、工具桥）
- [`EXPERIENCE_RUNTIME_AUTHORING.md`](EXPERIENCE_RUNTIME_AUTHORING.md) — 体验运行时能力包（checkpoint、recovery、agent run binding）
- [`MEMORY_PACKAGE_AUTHORING.md`](MEMORY_PACKAGE_AUTHORING.md) — 记忆 / 知识能力包

## 平台扩展

- [`SHARING_DISTRIBUTION.md`](SHARING_DISTRIBUTION.md) — 分享与分发：composition bundle、package-set lockfile、AI disclosure
- [`STORAGE_BACKEND_NEUTRALITY.md`](STORAGE_BACKEND_NEUTRALITY.md) — backend-neutral 存储契约与官方实验室
- [`POSTGRES_TDB_INTEGRATION.md`](POSTGRES_TDB_INTEGRATION.md) — PostgreSQL（事件后端）+ TDB（检索 provider）接入
- [`EXTERNAL_PROJECT_OPERATING_PLANE.md`](EXTERNAL_PROJECT_OPERATING_PLANE.md) — 外部项目操作平面（intake / workspace / adapter）
