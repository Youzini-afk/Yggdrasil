# Creative Inference Capability Alpha

> [English](./CREATIVE_INFERENCE_CAPABILITY_ALPHA.en.md) · [中文](./CREATIVE_INFERENCE_CAPABILITY_ALPHA.md)

## 立场

Yggdrasil 的近期交付路径是 **API-first**，因为大多数玩家和创作者今天会通过云端 API 获得模型能力。但 Yggdrasil 不能变成 **API-shaped**：HTTP、Bearer token、JSON schema、OpenAI/Anthropic/Gemini 请求形态都只能是 cloud adapter package 的内部细节，不能成为平台世界观。

架构原则：

- 云 API 是第一批适配目标，不是平台原语。
- `official/model-provider-lab` 是普通 cloud API adapter lab，不是 Yggdrasil 的模型抽象。
- 不继续用 provider 数量证明平台能力；八家云 provider 已足够证明现实 API 接入路径。
- 下一步证明的是：推理如何参与 Yggdrasil 的 session / branch / proposal / inspection / fork 创作运行时。
- 为 local/self-host/non-HTTP 保留 seam，但不在当前阶段做完整本地模型平台。

## 非目标

- 不做 LiteLLM / OneAPI 式中转站。
- 不做用户余额、计费、渠道后台、provider marketplace。
- 不做模型下载器、权重缓存、GPU 调度、llama.cpp/vLLM/Ollama 全量集成。
- 不新增 `kernel.model.*`、`kernel.prompt.*`、`kernel.chat.*`、`kernel.embedding.*`。
- 不把统一 chat schema、messages/system/user/assistant 或 OpenAI-compatible request 作为平台公共协议。

## Phase C0 — ADR 与计划（已完成）

交付：

- 本临时计划。
- `NEXT_STEPS` / `ALPHA_STATUS` 指向 Creative Inference Capability Alpha。
- 明确 “API-first but not API-shaped” 与 cloud adapter 降级原则。

验收：

- 文档链接通过。
- 无新增 kernel model/prompt/chat 术语作为协议或代码方法。

## Phase C1 — Transport-neutral inference capability contract（已完成）

目标：定义普通 package/capability 层的推理契约，不进入 kernel。

已交付：

- `sdk/typescript/inference-capability`：transport-neutral envelope 与 stream/error/capability manifest helpers。包含 `InferenceRequest`/`InferenceResponse`/`InferenceStreamFrame`/`InferenceError` 类型、`createInferenceRequest`/`validateInferenceRequest`/`classifyInferenceError`/`InferenceStreamLifecycle`/`createProviderCapabilityManifest`/`validateProviderCapabilityManifest` 辅助函数，69 项纯 TS 自测通过。
- `docs/guides/INFERENCE_CAPABILITY_AUTHORING.md` 与 `.en.md`：包作者指南。
- contract 明确不包含 URL/header/status code/OpenAI messages 作为必需字段。
- 错误分类覆盖 cloud（authentication/rate_limit/billing/provider_overloaded/…）和 local/resource（local_process_failed/local_resource_exhausted/local_model_not_loaded/…）两类错误。
- Provider capability manifest 支持 modality/transport/secret/network/local runtime hints。

契约表达：

- operation id / operation kind；
- input artifacts 或 opaque input payload refs；
- streaming / non-streaming；
- deadline / cancellation；
- resource hints；
- secret refs；
- transport kind hint（`http`、`local_process`、`in_memory`、`ipc`、`websocket`、`remote`、`custom`）；
- canonical stream frames；
- transport-neutral error taxonomy。

## Phase C2 — Non-HTTP fake local provider proof（已完成）

目标：证明 inference pipeline 不依赖 HTTP、Bearer、JSON provider schema。

已交付：

- `packages/official/inference-local-lab`：deterministic non-HTTP fake inference provider。
- capabilities：`describe_capabilities`、`invoke`、`stream`、`explain_error`。
- `crates/ygg-runtime/src/inproc/inference_local_lab.rs`：in-process handler 注册。
- conformance：5 个具名用例证明无 URL、无 Authorization、无 HTTP status、无 provider schema 也能产生 deterministic stream frames。
  - `official.inference_local_lab_describe_capabilities`：不需要 network/secret，transports include in_memory/local_process。
  - `official.inference_local_lab_invoke`：non-HTTP invoke 成功，无 URL/header/status/messages 字段，network_performed=false。
  - `official.inference_local_lab_invoke_rejects_http`：http transport 被拒绝，HTTP-shaped 和 messages-shaped 字段被拒绝，raw secret 被拒绝。
  - `official.inference_local_lab_stream`：deterministic start/chunk/progress/end frames，无 URL/header/status/provider_schema。
  - `official.inference_local_lab_explain_error`：覆盖 local/resource 错误类。

这不是本地大模型平台，只是防止抽象硬化为 HTTP proxy 的 seam proof。

## Phase C3 — Cloud adapter package reposition

目标：把现有 `official/model-provider-lab` 降级成 cloud adapter，而不是平台抽象。

交付候选：

- 文档和 manifest 描述改为 cloud API adapter lab。
- `MODEL_PROVIDER_INTEGRATION` guide 加 negative claims：不是 Ygg model abstraction、不是 API gateway、没有 kernel privilege。
- `normalize_request` 只描述为 package-local adapter helper，不作为平台 canonical schema。
- conformance wording 从 “model provider abstraction” 改为 “cloud adapter coverage”。

## Phase C4 — Ygg-native inference proposal vertical slice

目标：证明推理不是“prompt -> text response”，而是参与 Yggdrasil 创作运行时。

交付候选：

- `packages/official/inference-playtest-lab` 或扩展 `inference-local-lab`。
- 流程：session state → inference capability → proposal → inspect → approve/reject → apply → branch/fork → replay/audit。
- 输出必须是 approval-gated proposal 或 package-owned events，不是 chat message。
- provider 可替换为 fake local 或 cloud adapter；vertical slice 不依赖公网。

## Phase C5 — Durable docs cleanup

目标：删除临时计划，把成果收敛到长期 guide、status、matrix、README。

交付：

- 删除本临时计划。
- 更新 `docs/guides/INFERENCE_CAPABILITY_AUTHORING.md`、`MODEL_PROVIDER_INTEGRATION.md`、`ALPHA_STATUS`、`NEXT_STEPS`、`CONFORMANCE_MATRIX`。
- 全量验证并 push。

## 风险控制

- Kernel 继续 content-free。
- 官方 inference/cloud packages 没有私有出站或路由特权。
- Cloud adapter 继续可用，但不主导平台抽象。
- 非 HTTP proof 必须足够小，避免提前做完整本地模型系统。
