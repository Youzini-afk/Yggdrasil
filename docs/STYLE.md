# 文档体例与红线

> [English](./STYLE.en.md) · [中文](./STYLE.md)

这份文档是 Yggdrasil 仓库写文档的最低规则，目的是让文档为读者写，而不是被开发记录污染。

## 写给读者，不写给开发记录

文档的主要读者是新工程师和外部使用者，他们关心 **它是什么 / 怎么用 / 边界是什么**，不关心我们这一阶段做了什么。

应该写：

- 平台是什么、内核怎么定义、能力包是什么、项目是什么、surface 是什么。
- 怎么跑、怎么写一个能力包、怎么挂一个 surface、怎么管 secret。
- 边界在哪里、什么不在内核范围、什么走能力包、什么走项目。

不应该写：

- 「我们最近做了 X」、「Round 10A.4 完成了 Y」、「Phase B 优化把 Z 推进到了 …」。
- 把 commit message 风格的开发增量叙事直接当作文档章节。
- 把已经实现完毕的内容写成「正在推进的工作」。

## 不写阶段编号

仓库文档里不再使用 `Round X` / `Phase Y` / `Alpha Z` / `Beta N` / `T-track` / `U-track` 之类的阶段命名。这些都是开发过程中临时分组的名字，对读者没有意义。

替代写法：

- 已经在仓库里的能力 → 直接陈述事实，「内核暴露 X」「能力包提供 Y」「surface 走 Z」。
- 还没做的 → `planned`、`deferred`、`待补齐`、`后续工作`、`计划中`。
- 已完成但还要继续打磨 → `partial` / `partial-real` / `partial-opt-in`，并配合具体 delta 描述。

不要给「现在的状态」编序号，也不要保留过期阶段路标。

## 状态文档 vs 概念文档

两类文档分层，叙事方式不同：

- **概念文档**（`CHARTER`、`VISION`、`ARCHITECTURE`、`PLATFORM_KERNEL`、`CAPABILITY_PACKAGE`、`PLAY_CREATION_MODEL`、`KERNEL_V1_CONTRACT`、各个 guide）讲不变量、机制、契约、用法。这些不该被时间污染——除非平台立场或机制本身变了，否则不要更新它们。
- **状态文档**（`ALPHA_STATUS`、`roadmap/NEXT_STEPS`、`spec/CONFORMANCE_MATRIX`、`COMPATIBILITY_MATRIX`）讲当前状态、partial、deferred、下一步。这些是活文档，里面允许有数字、表格和实现进度，但仍不该写成开发日志。

如果一个 PR 把概念文档改成「最近又新增了 X 阶段」，方向就错了——应该让 X 进入状态文档，把概念文档的相关章节改成稳定描述。

## ZH/EN 1:1 对齐

主叙事、主导航、主指南必须同时维护中文版和英文版：

- 文件命名：中文是 `xxx.md`（默认），英文是 `xxx.en.md`。
- 文件顶部第二行是双语 blockquote：`> [English](./xxx.en.md) · [中文](./xxx.md)`，可在两种语言间切换。
- 改一份必须同步另一份；不允许只改一种语言后留 drift。
- 例外：`inventory/*.raw.md` 这类机器读文档以英文 ST 源码字面量为主，不做中文镜像；npm/cargo 风格的 package / SDK README 使用英文内容、不做镜像，符合相应生态惯例。

## 文档红线

写文档时不要做这些：

- ❌ 给文档起 `ROUND_X_PLAN.md` / `PHASE_Y_DESIGN.md` / `*_ALPHA.md` 这类阶段命名。临时计划文档要在工作完成后立即删除，长期文档收敛进 README / 对应 guide / 状态文档。
- ❌ 把 raw stderr / raw API key / raw secret 内容贴进文档。如果需要示例，使用 `secret_ref:env:NAME` 这种引用形式。
- ❌ 把 host 绝对路径（如 `/home/<user>/...`）写进面向读者的指南。可以写 `~/.yggdrasil/<area>/`，但不要暴露具体机器细节。
- ❌ 在文档里宣称「全域字节级 ST 对齐」「内核兼容 SillyTavern」之类未经 fixture 与对齐测试支撑的覆盖率结论。
- ❌ 在主叙事里堆砌「Round X / Round X+1 ...」一长串完成阶段。完成阶段属于 git history。
- ❌ 把 YdlTavern 等接入项目的语义（chat、character、tavern、prompt 等）写进 Yggdrasil 平台/内核文档。

## 临时计划文档的生命周期

计划性文档（例如重构、合并、清扫的 work plan）允许放在 `docs/roadmap/` 下，但必须满足：

- 文件顶部清楚标注「这是临时计划，完成后会删除」；
- 计划完成后立刻删除，不在仓库里残留过期路标；
- 长期价值的内容（最终架构决策、稳定边界）写进 architecture / spec / guide / status 文档，而不是留在计划文档里。

## 写之前先问

新增或重大改动文档前，先问自己：

1. 读者是谁？他想理解什么？
2. 这是概念还是状态？
3. 是否会增加冗余？同一个事实是否已经在 README、ALPHA_STATUS、guide 等地方表达过？
4. 是否会引入阶段编号、开发增量、过期路标、绝对路径、raw secret？
5. 中英文是否同步？

回答能让你写得更短、更稳。

## 一句话总结

**文档是给读者用的稳定参考，不是开发过程的日志归档。**
