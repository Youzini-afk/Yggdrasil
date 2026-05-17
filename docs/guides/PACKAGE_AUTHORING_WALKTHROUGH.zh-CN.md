# 能力包创作 walkthrough

> [English](./PACKAGE_AUTHORING_WALKTHROUGH.md) · [中文](./PACKAGE_AUTHORING_WALKTHROUGH.zh-CN.md)

这份 walkthrough 创建一个第三方能力包：它会出现在 Home，贡献 Forge 与 assistant surfaces，通过本地 conformance，并且可以与其他包 composition。它刻意使用与官方包相同的公开 manifest/capability/surface 路径。

## 1. 生成能力包

```bash
cargo run -p ygg-cli -- init-package /tmp/ygg-seed-package \
  --id example/seed-package \
  --entry subprocess \
  --language typescript-experience
```

生成的 manifest 包含：

- 一个面向 Home 的 `experience_entry` surface；
- 一个 `play_renderer` surface；
- 一个 `forge_panel` surface；
- 一个 `assistant_action` surface；
- 一个通过 subprocess JSON-RPC 暴露的 echo capability。

## 2. 本地验证能力包

```bash
cargo run -p ygg-cli -- package check /tmp/ygg-seed-package/manifest.yaml
cargo run -p ygg-cli -- package conformance /tmp/ygg-seed-package/manifest.yaml
```

这些命令只检查 manifest，并通过普通 capability 路径调用能力包。它们不会授予私有 host 访问权。

## 3. 创建 composition descriptor

```bash
cargo run -p ygg-cli -- init-composition /tmp/ygg-seed-composition --id example/seed-package
cargo run -p ygg-cli -- composition check /tmp/ygg-seed-composition/composition.yaml
```

composition descriptor 描述哪些包提供可启动入口、必须有哪些 surface slots。它不是内核里的 `game` 或 `experience` 类型。

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

## 5. 与官方包对比

`packages/official/` 下的官方包是 reference implementations，不是特权路径：

- `official/composition-lab` 解释 launch plans 与 surface graphs。
- `official/asset-lab` preview assets 并草拟 import plans。
- `official/projection-lab` 解释 projection rebuilds 与 source events。
- `official/playable-seed` 证明 reference playable package。

只要第三方包暴露兼容的 surfaces 与 capabilities，就应该能替换其中任意一个。

## 不变量

- Packages 不能自我声明 caller identity。
- Packages 只能写入授权 namespace。
- assistant-like packages 必须返回 proposals 或 events，不能直接修改可信状态。
- UI 和 tooling 只能使用公开 protocol methods。
- 如果 capability 需要 mutation，应通过权限检查；需要用户审批时走 `kernel.proposal.*`。
