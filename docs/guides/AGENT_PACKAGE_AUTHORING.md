# Agent 能力包创作指南

> [English](./AGENT_PACKAGE_AUTHORING.en.md) · [中文](./AGENT_PACKAGE_AUTHORING.md)

本文说明如何在 Yggdrasil 中编写 agent-like 能力包。核心原则是：**agent 是能力包语义，不是 kernel 语义**。

## 你应该使用什么

- 用普通 manifest 声明 agent-like capabilities。
- 用 `kernel.capability.invoke` 或 `kernel.capability.stream` 启动运行。
- 用 `kernel.capability.cancel` 取消 streaming invocation。
- 用 `kernel.proposal.*` 产出、审批和应用变更。
- 用 package-owned events 或 stream frames 记录 trace。
- 用 surface contributions 暴露 `assistant_action`、`forge_panel` 或 `home_card`。
- 用 `secret_ref` 而不是 raw secrets。
- 用显式 `provider_package_id` 处理 provider 冲突。

## 你不应该使用什么

- 不新增或依赖 `kernel.agent.*`、`kernel.model.*`、`kernel.prompt.*`、`kernel.memory.*`、`kernel.turn.*`。
- 不把 agent 直接写入 kernel state。
- 不让 agent 直接修改可信 asset/projection/session 状态；先生成 proposal。
- 不通过 tool bridge 借用其他包的权限。
- 不自动选择 official provider。
- 不在 trace、proposal、event、audit 或 error 中保存 raw secrets。
- 不默认提供 bash/read/write/edit 这类 coding-agent 工具。

## 从模板开始

生成 deterministic/no-network agent runtime 包：

```bash
cargo run -p ygg-cli -- init-package /tmp/ygg-agent \
  --id example/agent-runtime \
  --entry subprocess \
  --language typescript \
  --template agent-runtime
```

模板会生成：

- `example/agent-runtime/run`：streaming run capability。
- `example/agent-runtime/explain-run`：解释运行 trace。
- `example/agent-runtime/draft-proposal`：生成 approval-gated proposal draft。
- `example/agent-runtime/echo`：本地 conformance 兼容能力。
- `assistant_action` 和 `forge_panel` surfaces。
- no-network、no-real-model、no raw secret 的默认实现。

验证生成包：

```bash
cargo run -p ygg-cli -- package check /tmp/ygg-agent/manifest.yaml
cargo run -p ygg-cli -- package conformance /tmp/ygg-agent/manifest.yaml
```

## 使用 `ygg-agent-adapter` SDK

`sdk/typescript/ygg-agent-adapter` 是薄 adapter，不是完整 agent framework。它用于：

- 把 Ygg capability descriptor 映射为 pi-style tool descriptor。
- 构造 `kernel.capability.invoke` / `kernel.capability.stream` request payload。
- 生成 package-owned trace event payload。
- 生成 approval-gated proposal draft payload。
- 做 provider ambiguity、permission preview 和 raw-secret blocking 诊断。

示意：

```ts
import { createYggAgentAdapter } from "../../sdk/typescript/ygg-agent-adapter/index.js";

const adapter = createYggAgentAdapter({
  protocolClient,
  packageId: "example/agent-runtime",
});

const tool = adapter.createCapabilityTool({
  capability_id: "example/tool/plan",
  provider_package_ids: ["example/tool"],
  streaming: false,
});

const plan = await adapter.invokeCapabilityTool(tool, {
  input: { topic: "safe plan" },
  provider_package_id: "example/tool",
});
```

如果多个 provider 暴露同一能力，必须显式选择 `provider_package_id`。不要自动选择第一个 provider，也不要偏向 `official/*`。

## 官方参考包

`official/pi-agent-runtime-lab` 是普通参考包。它提供 deterministic/no-network：

- run plan
- trace summary
- proposal draft
- echo

它没有官方特权，不是真实 agent runtime，不做 model inference。

`official/capability-tool-bridge-lab` 也是普通包。它只生成 tool discovery、permission preview 和 invocation/streaming plans；它**不实际代替 agent 调用目标 capability**，避免 confused deputy。

## 第三方替换证明

参考：

- `examples/packages/thirdparty-agent-runtime/manifest.yaml`
- `examples/compositions/agent-runtime-replacement/composition.yaml`

这个例子证明第三方 agent runtime 可以提供同等 surface/capability/proposal/trace 形状，而 official 包只是 `replacement_candidate`，没有优先级。

验证：

```bash
cargo run -p ygg-cli -- package check examples/packages/thirdparty-agent-runtime/manifest.yaml
cargo run -p ygg-cli -- composition check examples/compositions/agent-runtime-replacement/composition.yaml
```

## UI 观察

Forge 的 Agent Observability section 和 Assist Drawer 的 Agent Readiness panel 只从 public protocol、surface contributions、capabilities、events 和 proposals 中提取信息。它们不 hardcode official 包，也不启动真实 agent/model。

## 与 pi 的关系

`/workspace/Yggdrasil/pi` 是参考来源：

- `pi-agent-core` 的 event/tool/gate/queue 思路可被普通包内部吸收。
- `pi-ai` 的 faux provider / stream shape 可作为未来 model package 参考。
- `pi-coding-agent` 只作为产品和观测经验参考，不嵌入 Yggdrasil。

更多边界见 [`../architecture/PI_INTEGRATION.md`](../architecture/PI_INTEGRATION.md) 和 [`../../integrations/pi/README.md`](../../integrations/pi/README.md)。
