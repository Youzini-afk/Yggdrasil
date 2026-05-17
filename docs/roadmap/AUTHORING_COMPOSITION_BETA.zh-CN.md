# Authoring & Composition Beta+

> [English](./AUTHORING_COMPOSITION_BETA.md) · [中文](./AUTHORING_COMPOSITION_BETA.zh-CN.md)

Authoring & Composition Beta+ 是 foundation、official labs、code-health split 和 runtime split 之后的下一条平台证明路线。TavernHeadless 已经启发了第一批 creative/model capability packages；这条路线把重心转向第三方 package 创作和可替换 composition。

目标不是做 Tavern clone、完整 Studio 或 game runtime。目标是让普通 package 路径真实可用：外部作者能通过与官方包相同的公开协议创建、运行、检查、组合和替换 packages。

## 目标

- 让 `ygg init-package` 支持多种 surface-slot package 形态。
- 增加不依赖 private runtime access 的本地 package fixture execution 和 diagnostics。
- 让 package reload/restart/log 开发循环可见且可重复。
- 强化 composition descriptors，使 package sets 能被明确检查、启动和替换。
- 把 Forge 改进成公开协议上的 package/composition authoring surface。
- 证明第三方 playable package 可以替换 official seed，而没有 official priority 或 kernel hardcoding。

## 非目标

- 不做 chat/message/turn runtime。
- 不做 live model inference。
- 不做 SillyTavern compatibility runtime。
- 不做 marketplace、package signing、dependency resolver 或 package registry service。
- 不给 Forge 私有 runtime backdoor。
- 不给官方包特权。

## Phase H1 — Authoring templates and fixture runner

把生成 package templates 从当前单一 experience template 扩展到多个 surface slot。

交付：

- `experience_entry`、`play_renderer`、`forge_panel`、`assistant_action` 和 `asset_editor` surface 的 template variants。
- 本地 fixture runner，使用 canned inputs 调用声明的 capabilities 并报告结构化结果。
- 生成 template variants 的 conformance。

## Phase H2 — Package development diagnostics and reload loop

让 package 开发循环可见且可重复。

交付：

- `package check` 的 manifest diff/diagnostic 输出。
- 通过现有公开 runtime 路径提供 dev-loop package restart/reload diagnostics。
- 为 generated 或 fixture packages 增加 package logs/status smoke 覆盖。

## Phase H3 — Composition descriptor v2 diagnostics

让 compositions 描述明确 package sets 与 replacement expectations。

交付：

- Composition descriptor 增加 title/description、optional packages、required capabilities、default activation、permission expectations、replacement candidates 和 compatibility notes。
- `composition check` 诊断 missing packages、surfaces、capabilities、entry activation、permission expectations 和 replacement candidates。
- `official/composition-lab` 能总结 launch plans、surface graphs、permission previews 和 replacement diagnostics。

## Phase H4 — Forge authoring surfaces

把 web shell 改进为诚实的公开协议 authoring/inspection surface。

交付：

- Forge 中的 package/surface/capability authoring panels，仅使用 public protocol data。
- Manifest/surface descriptor previews。
- 如果通过 assets/projections/capabilities 可获得，则显示 composition diagnostics。
- Proposal review 仍然 approval-gated。

## Phase H5 — Third-party replacement proof

新增非官方 package，证明 official seeds 可替换。

交付：

- 一个 third-party playable example package，拥有等价 launch/Play/Forge/Assist surface shape，但语义不同。
- 一个 composition 可以明确选择 official seed 或 third-party replacement。
- Conformance 证明 Home/Forge/Assistant-style discovery 和 capability invocation 不偏向 `official/*`。

## Phase H6 — Documentation and final validation

更新 durable guides 和 status docs，然后删除本 completed plan document。

必跑检查：

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

还要对代表性 official 与 third-party example packages 跑 package checks，并跑 doc-link check。

## 不变量

- Kernel 保持 content-free。
- Package authorship 使用 manifests、capabilities、surfaces、hooks、proposals 和 protocol calls。
- 官方包保持普通 package。
- Forge 和生成工具使用 public protocol paths，而不是 private runtime internals。
- TavernHeadless 保持 reference ledger，而不是 roadmap driver。
