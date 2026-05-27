# 平台壳 Descriptor 驱动改造（临时计划）

> 临时计划文档。完成后删除，并把长期状态收敛到 `ALPHA_STATUS.md`、`NEXT_STEPS.md`、`clients/web/README.md` 和 surface 贡献指南。

## 背景

用户提出：平台前端也应该是能力包的呈现，而不是全部写死在 `clients/web` 里。

调查结论：

- 项目层已经能换包换前端。YdlTavern 这类项目通过 `surface_bundle` 安装、解析、iframe 挂载。
- 平台主壳还不能换。Home、Settings、Workshop、Quick Actions 都是直接 React 代码。
- `kernel.v1.surface.contribution.list` 已存在，但 Web 端几乎没有消费。
- `home_card` 已在 23 个 official 包中声明，但这些声明目前不进入 Home。
- 当前 `SurfaceSlot` 没有 `quick_action` 或 `workshop_card`。

目标不是把整个 Home 变成插件系统，而是先让平台壳能安全消费小颗粒、结构化的包贡献。

## Oracle 复核后的收窄结论

第一版只做三个入口：

- `quick_action`
- `workshop_card`
- 带 `shell_schema_version` 的 `home_card` 试点消费

推迟：

- `update_item`：它更像 package lifecycle / advisory / migration state，不应作为静态 surface slot 首批进入。
- `settings_link`：Settings 是 trusted shell 控制面，首批不开放。

同时明确：

- 不做 host-side i18n 解析。
- 不在 Home 展示 invalid descriptor。
- 不批量消费旧的 23 个 `home_card`。
- 不引入 trusted inline package。
- 不让包贡献一键调用任意 capability。
- 保留平台内置 4 个 quick action。

## 设计原则

1. **两层 surface 模型并存**
   - 结构化 descriptor：包只贡献受限元数据，平台壳渲染。无 JS、无 iframe、无 bundle。
   - iframe surface bundle：YdlTavern / 项目体验 / 复杂 surface 保持现状。
2. **平台拥有渲染器**。包只能给 title、description、icon hint、order、target 等数据。
3. **失败不影响 Home**。descriptor 错误进入开发者诊断，不进入用户 Home。
4. **官方包没有特权**。官方和第三方走同一个 manifest、package check、contribution.list。
5. **Home 不整体可替换**。Continue Card、Project Grid、Activity Timeline、Topbar、Auth、Settings 核心 tab 保持 trusted shell。
6. **descriptor 不是权限**。点击入口不能绕过 capability permission、proposal、redaction、audit。

## 本期范围

### 新增 slot

| slot | 用途 | 首批能力 |
|---|---|---|
| `quick_action` | Home 工作台快捷入口 | 追加到平台内置 action 之后 |
| `workshop_card` | Workshop 工具/模板/示例入口 | 平台渲染小卡 |

### 复用 slot

| slot | 用途 | 首批能力 |
|---|---|---|
| `home_card` | Home capability 试点卡 | 只消费带 `shell_schema_version` 的新式 descriptor |

### 明确不做

- `update_item`
- `settings_link`
- `settings_panel`
- `topbar`
- `activity_item`
- 每张卡一个 iframe
- trusted inline package
- 任意 URL / HTML / Markdown / SVG / package icon resource

## Descriptor metadata v1

`SurfaceContribution.metadata` 仍是 `serde_json::Value`，但 Web shell 只消费带 `shell_schema_version: 1` 的 metadata。

Host / package check 做安全形状校验；Web 做 locale fallback、icon fallback 和布局过滤。

### 通用字段

```yaml
metadata:
  shell_schema_version: 1
  title:
    zh-CN: "新建角色"
    en: "New persona"
  description:
    zh-CN: "创建或导入一个 persona 草稿。"
    en: "Create or import a persona draft."
  icon_hint: "user"
  order: 20
```

规则：

- `title` 必须是对象，至少有 `en` 或 `zh-CN` 之一。
- `description` 可选。
- Web 根据当前 locale 解析：当前语言 → `en` → `zh-CN` → 第一个可用文本。
- Host 不解析当前语言。
- `icon_hint` 是短字符串，不是 URL。Web 用白名单映射，未知 icon fallback。
- `order` 是排序提示，稳定 tie-break：`order` → `package_id` → `surface.id`。
- 文本字段限制长度，禁止 HTML/Markdown 渲染。

### `quick_action`

```yaml
slot: quick_action
capability_id: official/project-intake-lab/draft_intake_plan # 可选，首批只允许同包 capability
metadata:
  shell_schema_version: 1
  title: { zh-CN: "添加外部项目", en: "Add external project" }
  description: { zh-CN: "生成外部项目 intake 计划。", en: "Draft an external project intake plan." }
  icon_hint: "plus"
  order: 30
```

首批点击语义：

- 没有 `capability_id` / `surface_id`：渲染为 disabled，并在开发者诊断可见。
- 有 `surface_id`：只能指向同包 surface。
- 有 `capability_id`：只能指向同包 capability，且必须走现有 permission / proposal / audit；不能直接用 Web host 权限静默执行。
- 不支持跨包 capability。
- 不支持远程 URL。

### `workshop_card`

```yaml
slot: workshop_card
metadata:
  shell_schema_version: 1
  title: { zh-CN: "组合检查", en: "Composition check" }
  description: { zh-CN: "检查组合描述符和 surface wiring。", en: "Check composition descriptors and surface wiring." }
  icon_hint: "stack"
  order: 40
  category: "tool"
```

规则：

- `category` 只作为显示分组 hint：`tool | template | example`。
- 总数有限。首批 Home 最多显示 6 张，剩余折叠或隐藏。
- 不允许营销型长文案。

### `home_card`

首批只消费显式带 `shell_schema_version: 1` 的 `home_card`。旧的 23 个 official `home_card` 不自动进入 Home。

```yaml
slot: home_card
metadata:
  shell_schema_version: 1
  title: { zh-CN: "项目接入", en: "Project intake" }
  description: { zh-CN: "从 Git 或本地项目创建 intake 草案。", en: "Create an intake draft from a Git or local project." }
  icon_hint: "folder"
  order: 20
```

规则：

- Home card 试点数量上限 3。
- 不改变 Continue Card 尺寸。
- 不进入 Project Grid。

## 协议与校验

### 改动点

- `crates/ygg-core/src/manifest.rs`：新增 `QuickAction`、`WorkshopCard`。
- `crates/ygg-cli/src/commands/package.rs` 和 conformance 中的穷尽匹配同步增加分支。
- `export-schemas` / `generate-sdks` 重生成 schema、Rust SDK、TypeScript SDK、OpenAPI。
- package check 增加 shell descriptor metadata v1 校验。

### 校验规则

Host / package check 应校验：

- metadata 是对象。
- `shell_schema_version` 是 `1`。
- `title` 至少有一个合法 locale 文本。
- 文本长度限制。
- `icon_hint` 是短字符串，不能是 URL / HTML / path。
- `capability_id` 若存在，必须属于同一个 package。
- `surface_id` 若存在，必须属于同一个 package。
- 每个 package 每个 slot 的贡献数量上限。

用户 Home 不渲染 invalid descriptor。错误进入 package check、diagnostics 或 installed package 详情。

## Web 改动

### Hook

新增 `useSurfaceContributions(slot)`：

- 调 `client.surfaceContributions(slot)`。
- 失败 fallback `[]`。
- 不依赖不存在的 package lifecycle event tail。
- install / uninstall / package refresh 完成后由调用方显式 refresh。
- route focus 时可轻量刷新。

### 解析器

新增 `shell-contributions.ts`：

- `parseShellContribution(contribution, slot, locale)`。
- 过滤无 `shell_schema_version` 的旧 descriptor。
- invalid descriptor 返回 `null`，开发模式可 console warn，生产不渲染。
- icon fallback 在 Web 端做。

### 渲染器

| 渲染器 | 作用 |
|---|---|
| `QuickActionList` | 平台内置 + package quick actions |
| `WorkshopCardList` | tool/template/example cards |
| `HomeCapabilityCards` | 最多 3 张 schema-versioned home cards |

### Home 接入

- 平台内置 4 个 quick action 保留：install / data folder / settings / profile。
- package quick actions 追加，数量上限。
- Workshop 保留 Disk Usage，新增 package cards。
- `updates` 暂不做 descriptor-driven。
- Continue Card、Project Grid、Activity Timeline 不动。

## 包改动

第一批不批量补 23 个 official 包。

只选试点：

| 包 | contribution |
|---|---|
| `official/project-intake-lab` | `quick_action` + `home_card` metadata v1 |
| `official/composition-lab` | `workshop_card` metadata v1 |
| `official/schema-tools` | `workshop_card` metadata v1 |
| `examples/packages/thirdparty-surface-fixture` | `quick_action` 测试样本 |

YdlTavern 后置验证：

- 若由 YdlTavern surface package 贡献，只能指向同包 surface。
- 不为 YdlTavern 开跨包 capability 例外。
- 不影响项目 iframe surface 挂载。

## 阶段

### Phase 0 — 计划收窄

- Oracle 复核。
- 按复核结论收窄本文档。
- 提交并推送。

### Phase 1 — Contract minimum

- 新增 `quick_action`、`workshop_card`。
- package check 增加 metadata v1 shape 校验。
- schema / SDK / OpenAPI 重生成。
- conformance 增加新 slot 过滤、metadata invalid、same-package ownership case。
- 提交并推送。

### Phase 2 — Web parser and renderers

- `useSurfaceContributions`。
- `shell-contributions` parser。
- icon registry。
- `QuickActionList`、`WorkshopCardList`、`HomeCapabilityCards`。
- Web 单元测试。
- 提交并推送。

### Phase 3 — Home integration

- Home 接入 quick action。
- Workshop 接入 card。
- Home 接入 schema-versioned home card pilot。
- @designer 复核布局：Continue Card 不变高、不压下方。
- 提交并推送。

### Phase 4 — Package pilots

- 给少量 official 包补 metadata v1。
- 给 third-party fixture 补 quick action。
- YdlTavern 如安全则补同包 surface quick action，否则只验证不改。
- package check 全过。
- 提交并推送。

### Phase 5 — E2E and review

- 安装 fixture → Home 出现 contribution。
- 卸载 fixture → contribution 消失或 refresh 后消失。
- YdlTavern 安装/打开/iframe mount 不回归。
- @oracle 安全复核。
- 修正问题并提交推送。

### Phase 6 — Documentation convergence

- 删除本文档。
- 更新长期 docs。
- 更新 Web README。
- 更新 package author guide / surface guide。
- 最终验证并提交推送。

## 验收标准

- `cargo test --workspace` 通过。
- `cargo run -p ygg-cli -- conformance` 通过。
- `./scripts/validate-schemas.sh` 通过。
- `npm run check --prefix clients/web` 通过。
- `npm test --prefix clients/web` 通过。
- `npm run build --prefix clients/web` 通过。
- Home 无 package contribution 时仍显示平台内置 4 个 quick action。
- invalid descriptor 不进入 Home。
- unknown icon fallback。
- locale fallback 在 Web 端完成。
- Continue Card 尺寸不回归。
- descriptor consumption 不调用 `resolve_bundle`，不 mount iframe，不 dynamic import package JS。
- official 与第三方贡献走同一校验路径。

## 后续可能的 RFC

- package advisory / update lifecycle，不作为 `SurfaceSlot`。
- Settings package detail link，不作为核心 tab 插件。
- 更丰富的 proposal action UX。
- package contribution cache invalidation event。
