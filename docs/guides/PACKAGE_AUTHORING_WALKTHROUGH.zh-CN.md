# 能力包创作 walkthrough

> [English](./PACKAGE_AUTHORING_WALKTHROUGH.md) · [中文](./PACKAGE_AUTHORING_WALKTHROUGH.zh-CN.md)

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
