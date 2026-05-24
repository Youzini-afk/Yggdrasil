# Docs Narrative Convergence (临时计划)

> 这是一份临时计划文档，完成后会在 D4 收尾时删除。

## 目标

把跟随开发随手写的文档收敛成「为读者写」的稳定叙事：
- 一个概念 = 一个权威位置；
- 删除 Round/Phase/Alpha 命名（保留时间感即可）；
- 状态/路线 ≠ 概念解释；
- 历史快照只保留有未来价值的；
- ZH/EN 1:1 对齐。

## 阶段

### D0 — 安全删除与计划

直接删除：

- `Yggdrasil/docs/product/EXPERIENCE_LED_PLATFORM_BETA.{md,en.md}`：内容已被 `PLAY_CREATION_MODEL.md` 与 `ARCHITECTURE.md` 覆盖，本身是过期 Beta 路线叙事。
- `YdlTavern/docs/research/round8/*`（6 个 .md）：Round 8 的研究快照，决策与契约已落入 `docs/guides/EXTENSION_COMPATIBILITY.md` / `docs/ARCHITECTURE.md`。
- `YdlTavern/docs/research/` 目录（在删除全部 round8 文件后清空）。

清理引用：

- Yggdrasil：`docs/roadmap/NEXT_STEPS.{md,en.md}`、`docs/product/README.{md,en.md}`、`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.{md,en.md}` 中对已删 BETA 文档的指向。
- YdlTavern：`docs/README.{md,en.md}`、`docs/guides/PERFORMANCE_BASELINE.{md,en.md}`、`docs/guides/EXTENSION_COMPATIBILITY.{md,en.md}` 中对 `research/round8/*` 的指向。

### D1 — Yggdrasil 顶层叙事重写

重写：`README.{md,en.md}`、`docs/README.{md,en.md}`、`docs/ALPHA_STATUS.{md,en.md}`、`docs/roadmap/NEXT_STEPS.{md,en.md}`、`docs/spec/CONFORMANCE_MATRIX.{md,en.md}`。

合并：`integrations/model-providers/{README,new-api-ledger,tavern-headless-ledger,error-taxonomy,stream-compatibility}.md` → 一份主 ledger。

清扫 Round/Phase 命名：在以下文档中改成稳定语言（不删事实）：

- `docs/guides/PACKAGE_INSTALLATION`、`PROJECT_MODEL`、`REAL_MODEL_END_TO_END`、`CONFORMANCE_KIT`
- `docs/spec/KERNEL_V1_CONTRACT`、`docs/spec/v1/LOCKFILE_FORMAT`
- `docs/performance/BASELINE`、`PERFORMANCE_AND_CODE_HEALTH`
- `docs/protocol/PROTOCOL_V0`、`docs/architecture/ARCHITECTURE`
- `integrations/pi/README`、`integrations/pretext/README`、`integrations/tavern-headless/README`

### D2 — YdlTavern 顶层叙事重写

重写：`README.{md,en.md}`、`docs/README.{md,en.md}`、`docs/ARCHITECTURE.{md,en.md}`、`docs/COMPATIBILITY_MATRIX.{md,en.md}`、`docs/roadmap/NEXT_STEPS.{md,en.md}`。

简化 tracks：`docs/tracks/{B..I}_*.{md,en.md}` 改成「当前推进面 + 下一步」，删历史 Round 长篇回顾。

合并 golden harness 说明：`golden-harness/README.md` 与 `docs/guides/GOLDEN_HARNESS.{md,en.md}` 边界划清。

清扫命名：`EXTENSION_COMPATIBILITY`、`PERFORMANCE_BASELINE`、`UI_FORK_GUIDE`、`E2E_INTEGRATION` 中的 Round/Phase 字样。

### D3 — 双语对齐 + 链接 + 新人路径

- 检查所有 markdown 链接不死链；
- ZH/EN 1:1 对齐；
- `docs/README.{md,en.md}` 提供「新人 1/2/3」路径；
- `inventory/*.raw.md` 标注「机器读 / 维护者」。

### D4 — 文档体例与红线

- 新增 `docs/STYLE.{md,en.md}`：写文档的最低规则（不写 Round/Phase/Alpha、概念优先、ZH/EN 同步）；
- 删除本临时计划；
- 最终验证后 push。

## 不在范围

不冻结协议、不规划 release、不动 inventory 内容、不动 sdk/example README、不重写已经稳定的 architecture/spec/v1 概念部分。
