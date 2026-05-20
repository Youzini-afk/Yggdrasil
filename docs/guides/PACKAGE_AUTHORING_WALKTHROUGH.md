# 能力包创作 walkthrough

> [English](./PACKAGE_AUTHORING_WALKTHROUGH.en.md) · [中文](./PACKAGE_AUTHORING_WALKTHROUGH.md)

这份 walkthrough 创建一个第三方能力包：它会出现在 Home，贡献 Forge 与 assistant surfaces，通过本地 conformance，并且可以与其他包 composition。它刻意使用与官方包相同的公开 manifest/capability/surface 路径。

## 1. 生成能力包

```bash
cargo run -p ygg-cli -- init-package /tmp/ygg-seed-package \
  --id example/seed-package \
  --entry subprocess \
  --language typescript \
  --template full-surface
```

生成的 manifest 包含：

- 一个面向 Home 的 `experience_entry` surface；
- 一个 `play_renderer` surface；
- 一个 `forge_panel` surface；
- 一个 `assistant_action` surface；
- 一个 `asset_editor` surface；
- 一个通过 subprocess JSON-RPC 暴露的 echo capability。

如果只需要更窄的包，可以选择其他 template：

```bash
cargo run -p ygg-cli -- init-package /tmp/ygg-assist \
  --id example/assist \
  --entry subprocess \
  --language typescript \
  --template assistant-action

cargo run -p ygg-cli -- init-package /tmp/ygg-asset-editor \
  --id example/asset-editor \
  --entry subprocess \
  --language python \
  --template asset-editor
```

可用 templates：

- `basic` — 只有 capability，没有 surfaces。
- `experience` — 只有 Home `experience_entry`。
- `play-renderer` — Play renderer surface。
- `forge-panel` — Forge panel surface。
- `assistant-action` — assistant action surface，带 approval policy metadata。
- `asset-editor` — asset editor surface。
- `full-surface` — 所有 authoring/play surface slots。
- `networked` — 带 declared network permissions（`host`、`methods`、`purpose`）的网络能力包，使用 `secret_ref`，带 outbound audit helper。无 raw secrets、无隐式 network 访问。演示 `sdk/typescript/secure-execution` 中的 `NetworkDeclaration` 和 `OutboundAuditHelper`。
- `streaming` — 带 faux frame 生命周期的 streaming capability（`StreamFrameClient`）。演示 `start`/`chunk`/`end` frame 和 `redaction_state`。不做真实 model inference。使用 `sdk/typescript/secure-execution`。
- `agent-runtime` — deterministic/no-network agent-like subprocess 包。包含 streaming `run` capability、`explain-run` trace summary、`draft-proposal` approval-gated proposal、`echo` capability，以及 `assistant_action` + `forge_panel` surfaces。使用 `StreamFrameClient`（`sdk/typescript/secure-execution`）与 `createTraceEvent`/`createProposalDraft`/`blockRawSecrets`（`sdk/typescript/ygg-agent-adapter`）。不做真实 model inference、不出网、不暴露 raw secret。
- `experience-runtime` — deterministic/no-network experience-runtime subprocess 包。包含 `describe-contract`、`create-checkpoint`、`inspect-checkpoint`、`draft-recovery`、`bind-agent-run` 和 `echo` capabilities，以及全部四个 experience surfaces。使用 `sdk/typescript/experience-runtime` SDK。不做真实 model inference、不出网、不暴露 raw secret。
- `playable-board` — deterministic/no-network playable board subprocess 包。包含 `launch`、`project_state`、`render_payload`、`record_player_action`、`request_change`、`create_checkpoint` 和 `echo` capabilities，以及全部四个 experience surfaces。最接近 `official/playable-creation-board` 形态的第三方创作者模板。不做真实 model inference、不出网、不暴露 raw secret。
- `playable-experience` — deterministic/no-network playable experience subprocess 包。包含 `playable-board` 的所有 capabilities 外加 `inspect_checkpoint` 和 `draft_recovery`，支持完整的 checkpoint/recovery 生命周期。全部四个 experience surfaces。不做真实 model inference、不出网、不暴露 raw secret。

`--language typescript-experience` 仍作为 legacy shortcut 保留，用于生成完整 experience-shaped package。

## 2. 本地验证能力包

```bash
cargo run -p ygg-cli -- package check /tmp/ygg-seed-package/manifest.yaml
cargo run -p ygg-cli -- package conformance /tmp/ygg-seed-package/manifest.yaml
cargo run -p ygg-cli -- package run-fixture /tmp/ygg-seed-package/manifest.yaml
cargo run -p ygg-cli -- package reload /tmp/ygg-seed-package/manifest.yaml
```

这些命令只检查 manifest，并通过普通 capability 路径调用能力包。它们不会授予私有 host 访问权。

`package check` 会打印 authoring diagnostics，例如 entry kind、trust level、capability count、按 slot 分组的 surfaces、permission summary、sandbox policy，以及无 capabilities 或无 surfaces 的 warnings。`package run-fixture` 用确定性 fixture input 调用声明的非 streaming capabilities，并输出结构化 JSON 结果。`package reload` 练习本地 load/restart/unload 循环，并报告 package status 与 logs。

## 3. 创建 composition descriptor

```bash
cargo run -p ygg-cli -- init-composition /tmp/ygg-seed-composition --id example/seed-package
cargo run -p ygg-cli -- composition check /tmp/ygg-seed-composition/composition.yaml
```

composition descriptor 描述哪些包提供可启动入口、必须有哪些 surface slots。它不是内核里的 `game` 或 `experience` 类型。

Composition descriptor v2 fields 还能声明 optional packages、required capabilities、permission expectations、replacement candidates、default activation metadata 和 compatibility notes。`composition check` 会报告已加载 package paths、按 slot 分组的 surfaces、capabilities、缺失的 required surfaces/capabilities、optional-package warnings 与 replacement diagnostics。

要查看 replacement proof，可以检查内置第三方 example：

```bash
cargo run -p ygg-cli -- package check examples/packages/thirdparty-playable-seed/manifest.yaml
cargo run -p ygg-cli -- composition check examples/compositions/playable-seed-replacement/composition.yaml
```

该 package id 是 `thirdparty/playable-seed`，不是 `official/*`，并且在没有 official priority 的情况下暴露兼容的 Play/Forging/Assistant/Asset surfaces。

## 4. 在 host profile 中加载能力包

把包 manifest 加入一个 host profile，例如：

```yaml
autoload:
  - /tmp/ygg-seed-package/manifest.yaml
```

然后运行：

```bash
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
```

Home 通过 `kernel.surface.contribution.list` 发现能力包。Forge 通过同一公开协议发现 panels。UI 不会获得私有 runtime handle。

Forge 现在包含基于 public protocol data 的轻量 authoring panels：

- 按 provider package 分组的 package 与 capability inventory；
- 按 slot 分组的 surface inventory；
- packages、capabilities、surfaces、assets、projections 与 entry surfaces 的 authoring diagnostics；
- templates、package checks、fixture runs、reloads 与 compositions 的 CLI command guidance。

## 5. 与官方包对比

`packages/official/` 下的官方包是 reference implementations，不是特权路径：

- `official/composition-lab` 解释 launch plans 与 surface graphs。
- `official/asset-lab` preview assets 并草拟 import plans。
- `official/projection-lab` 解释 projection rebuilds 与 source events。
- `official/playable-seed` 证明 reference playable package。

只要第三方包暴露兼容的 surfaces 与 capabilities，就应该能替换其中任意一个。

`examples/packages/thirdparty-playable-seed` package 是当前 proof。Conformance 会验证它的 surfaces 可发现、capabilities 通过普通 routing 调用、composition checks 通过，并且共享 capability id 在没有 explicit provider 时会被判定为 ambiguous。不存在隐式 official priority。

## 不变量

- Packages 不能自我声明 caller identity。
- Packages 只能写入授权 namespace。
- assistant-like packages 必须返回 proposals 或 events，不能直接修改可信状态。
- UI 和 tooling 只能使用公开 protocol methods。
- 如果 capability 需要 mutation，应通过权限检查；需要用户审批时走 `kernel.proposal.*`。

## 6. Secure execution helpers

`sdk/typescript/secure-execution` 模块为需要 secret references、网络声明、outbound audit 和 streaming frames 的包提供薄且协议安全的 helper。不暴露任何私有内核内部。

### Secret references

```ts
import { secretRef, isValidSecretRef, looksLikeRawSecret } from "../../sdk/typescript/secure-execution/index.js";

// 创建 secret reference（不要在 payload 中嵌入 raw secrets）
const ref = secretRef("env", "MY_API_KEY"); // → "secret_ref:env:MY_API_KEY"

// 验证
isValidSecretRef("secret_ref:env:KEY"); // true
isValidSecretRef("sk-abc123");           // false
```

### 网络声明

```ts
import { NetworkDeclaration } from "../../sdk/typescript/secure-execution/index.js";

const decl = new NetworkDeclaration({
  host: "api.example.com",
  methods: ["GET", "POST"],
  purpose: "model inference",
});
decl.toManifestEntry(); // manifest 兼容的对象
decl.matches("api.example.com", "POST"); // true
```

### Outbound audit helper

```ts
import { OutboundAuditHelper, secretRef } from "../../sdk/typescript/secure-execution/index.js";

const audit = new OutboundAuditHelper({
  packageId: "example/my-package",
  capabilityId: "example/my-package/fetch",
});
const payload = audit.buildRequestPayload({
  destinationHost: "api.example.com",
  method: "POST",
  secretRefsUsed: [secretRef("env", "MY_KEY")],
  purpose: "model inference",
});
// payload 只包含引用——永远不会有 raw secrets
```

### Stream frame client

```ts
import { StreamFrameClient } from "../../sdk/typescript/secure-execution/index.js";

const client = new StreamFrameClient();
const startFrame = client.start("example/stream/echo", {});
const chunk1 = client.chunk({ text: "faux token 1" });
const endFrame = client.end();
// Frame 包含 invocation_id、stream_id、sequence、redaction_state
```

## 7. No-network readiness proof

对于想证明自己已准备好在安全执行底座（secret refs、网络权限、streaming）上运行，但不想进行真实网络调用或 model inference 的包，可以参考内置示例：

```bash
cargo run -p ygg-cli -- package check examples/packages/faux-model-readiness/manifest.yaml
cargo run -p ygg-cli -- package check examples/packages/faux-agent-readiness/manifest.yaml
```

- `example/faux-model-readiness` 声明网络权限，使用 `secret_ref` 引用凭证，返回 discovery plans（非真实 API 响应），产生 faux streaming frames。不做真实 inference 或网络调用。
- `example/faux-agent-readiness` 仅产出 proposals/traces/plans，强调公开 protocol/capability/proposal 模式，无网络权限，产生 faux streaming trace frames。不连接 pi runtime 或 model inference。

这些包证明了 substrate shape，而不与任何特定 model 或 agent 实现耦合。

## 8. Playable package walkthrough — 从 template 到 playable

这份 walkthrough 展示一个新创作者如何只靠 docs、templates 和 Forge，在一天内从 template 到 playable package，不需要阅读 Yggdrasil 源码。

### 8.1 生成 playable board 包

```bash
cargo run -p ygg-cli -- init-package /tmp/my-playable-board \
  --id thirdparty/my-playable-board \
  --entry subprocess \
  --language typescript \
  --template playable-board
```

生成一个与 `official/playable-creation-board` 形态一致的包骨架：

- 4 个 experience surfaces：`experience_entry`、`play_renderer`、`forge_panel`、`assistant_action`
- 7 个 capabilities：`launch`、`project_state`、`render_payload`、`record_player_action`、`request_change`、`create_checkpoint`、`echo`
- 无 network 声明 — 默认 deterministic
- `package.ts` 包含每个 capability 的 deterministic/no-network stub

### 8.2 本地验证

```bash
cargo run -p ygg-cli -- package check /tmp/my-playable-board/manifest.yaml
cargo run -p ygg-cli -- package conformance /tmp/my-playable-board/manifest.yaml
cargo run -p ygg-cli -- package run-fixture /tmp/my-playable-board/manifest.yaml
cargo run -p ygg-cli -- package reload /tmp/my-playable-board/manifest.yaml
```

`package check` 现在输出面向创作者的诊断：

- **Experience surface coverage**：当 `experience_entry` 存在但缺少 `play_renderer`、`forge_panel` 或 `assistant_action` 时发出警告
- **Checkpoint/recovery capability coverage**：当 experience 包缺少 `create_checkpoint` 或 `draft_recovery` capability 时发出警告
- **危险 permissions**：对 wildcard `capabilities.invoke: ["*"]` 或空 methods 的 network 声明发出警告
- **Non-deterministic hint**：当请求 network 访问时发出警告（包默认不是 deterministic）

`package run-fixture` 现在在 capability 失败时提供针对性修复提示（如"检查 surface 的 capability_id 字段是否与提供的 capability 匹配"）。

`package reload` 现在在包 restart 后处于 degraded 状态时发出警告。

### 8.3 与其他包 composition

```bash
cargo run -p ygg-cli -- init-composition /tmp/my-board-composition --id thirdparty/my-playable-board
cargo run -p ygg-cli -- composition check /tmp/my-board-composition/composition.yaml
```

`composition check` 现在输出 experience 相关诊断：

- **Experience surface coverage**：显示哪些 surface slots 已覆盖或缺失
- **Replacement candidates**：显示声明的候选项及其加载状态
- **Replacement hint**：当多个包提供相同 slot 时，建议声明 `replacement_candidates`
- **State capability coverage**：显示 `create_checkpoint` 和 `draft_recovery` 的 provider 数量
- **Optional package coverage**：提示 `memory-lab` 和 `experience-observability-lab` 以获得更丰富的体验

### 8.4 与官方参考包对比

官方 `official/playable-creation-board` 包拥有相同的 4 surfaces 和 14 capabilities。你的第三方包使用相同的公开 manifest/capability/surface 路径 — 无特权、无特殊路由。两者同时加载时，内核不会优先选择官方包。若要在 composition 中替换它，将你的包声明为主要 provider，官方包声明为 `replacement_candidate`。

### 8.5 更丰富的生命周期：playable-experience template

如果你的 experience 需要 checkpoint 检查和恢复计划（中途保存/恢复、从故障中恢复），使用 `playable-experience` template：

```bash
cargo run -p ygg-cli -- init-package /tmp/my-playable-experience \
  --id thirdparty/my-playable-experience \
  --entry subprocess \
  --language typescript \
  --template playable-experience
```

这增加了 `inspect_checkpoint` 和 `draft_recovery` capabilities（共 9 个），支持完整的保存/检查/恢复生命周期。
