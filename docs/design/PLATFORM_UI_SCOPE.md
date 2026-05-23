# 平台 UI 范围与用户旅程

> [English](./PLATFORM_UI_SCOPE.en.md) · [中文](./PLATFORM_UI_SCOPE.md)

本文档列出 Yggdrasil 平台壳 (`clients/web`) 的 UI 范围、用户旅程、组件目录、
和按优先级排序的设计屏列表。视觉规则见 `PLATFORM_UI_DESIGN.md`。

---

## 范围边界

**做** — 平台壳的 UI:
- Home 项目货架
- Settings 多面板 (API Connections / Installed Packages / Profiles / About)
- Install 流程 (URL 输入、原生 vs 外部检测、wizard、进度)
- Project 框架 (mounted iframe 的上方 chrome)
- 通知 / Toast 系统
- Empty states / Error states
- Failure 详情 / 重试入口
- Cmd+K 命令面板 (后置)

**不做** — 项目内部 UI:
- YdlTavern 自己的整套界面 (保留 ST 风格 + 兼容社区扩展)
- 任何项目的 entry surface 内部
- 项目 surface 与平台壳的接缝由 surface-host iframe 边界处理, 项目自由发挥

---

## 用户旅程 (按重要性)

### J1: 首次启动 (clean install)

```
启动 Yggdrasil 桌面端
  → 平台壳启动, 加载 host
  → Home 路由空货架
  → 显示 first-run 引导
       核心信息: "你的工作台还是空的. 装一个项目开始."
       唯一行动: 大号 "+ 装入第一个项目" 卡片
       (可选) 一行小字提示 yg install 命令也可用
```

### J2: 装 YdlTavern

```
点 "+ 装入第一个项目"
  → Install Modal 打开
  → URL 输入框 (placeholder: github.com/user/repo 或 ./local/path)
  → 用户粘贴 github.com/Youzini-afk/Yggdrasil-Tavern, 回车
  → Modal 切换到 "解析中" 状态: 显示 git 探测进度
  → install-lab.detect_kind 返回 native (有 project.yaml)
  → Modal 切换到 "确认计划"
       项目: YdlTavern (signed: false)
       依赖: 2 个包 (官方+本仓)
       请求权限:
         - network: api.openai.com, api.anthropic.com, ...
         - secret: secret_ref:store:OPENAI_API_KEY 等
       签名状态: ⚠ unsigned (允许)
       Conformance: ✓ 通过
       [取消] [安装]
  → 用户点安装
  → Modal 切换到 "安装中": 进度条 + 当前步骤
       拉源 → 验整 → 写 store → 注册项目 → 完成
  → 安装完成: Modal 自动关闭
  → Toast: "YdlTavern 已安装" + 跳到 Home 操作
  → Home 货架现在有一张卡片: YdlTavern (Stopped)
```

### J3: 装外部项目 (无 project.yaml)

```
点 "+ 装入项目"
  → Install Modal, URL 输入
  → install-lab.detect_kind 返回 external
  → Modal 切换到 wizard:
      "这个仓库没有声明为 Yggdrasil 项目. 怎么用它?"
       [选项 A] 用 adapter 包装 — 让 Yggdrasil 把它包成一个能力包
                   (推荐当原项目是工具)
       [选项 B] 作为工作区打开 — agent 协助使用, 不包装
                   (推荐当你想自己探索)
       [取消]
  → 用户选 B
  → 后续步骤同 J2 (确认 → 安装 → 完成)
  → Home 货架显示卡片, type: external_workspace
```

### J4: 启动一个项目 (Play)

```
Home 货架 → 鼠标悬停 YdlTavern 卡片 (微 lift + 阴影加深)
  → 点 "▶ Play" 按钮
  → 卡片状态切换: Stopped → Starting (黄色 shimmer)
  → 后台: kernel.v1.project.start → 状态机转换 → 开 session
  → 卡片状态: Starting → Running (Aged Brass + 脉冲)
  → 路由切换到 Project frame
  → 上方 40px topbar 出现:
       [←返回] [项目图标] YdlTavern  [• Running]    [Stop] [⋯]
  → topbar 下方整个视口给项目 iframe
  → YdlTavern 自己的 UI 渲染 (ST DOM fork)
  → 用户开始使用项目
```

### J5: 配置 API Key

两个入口:

**入口 A — 项目内** (YdlTavern 已有, 不重做):
```
项目内 API Connections 抽屉 → 粘贴 → 保存
  → 选范围: Platform-wide / This project only
  → secret-store-lab.put_secret 或 put_project_secret
```

**入口 B — 平台 Settings** (新, 平台壳要做):
```
topbar 齿轮图标 → Settings 路由
  → 左侧 nav: API Connections / Installed Packages / Profiles / About
  → 选 "API Connections" 面板
  → 显示已存的 secrets 列表 (名字, 不显示值, 创建时间, 关联项目数)
  → "+ 添加新密钥" 按钮
  → 弹出小对话框: provider 选择 / 名字 / 值 / 范围
       范围: Platform / 选定的 project (如果有运行中的项目)
  → 保存 → secret-store-lab.put_secret
  → 列表刷新
  → 用户能编辑、删除某项 secret
  → 删除时弹确认对话框
```

### J6: 管理已装包

```
Settings → Installed Packages
  → 已装项目和包列表
       每行: 包 ID (mono) | version | source (git/local/internal) | state | 占用空间 | 最近更新
  → 顶部过滤: 全部 / 仅项目 / 仅依赖包 / 按状态
  → 行内 actions: ⋯ 菜单 (Update / Uninstall / View permissions / View logs)
  → 点 Update: 检查上游 → 显示 changelog → 确认升级
  → 点 Uninstall:
       如果是项目, 弹出 keep-data / delete-data 选择 (per Round 10A.2)
       如果是依赖包, 警告其它项目可能受影响
  → 点 View permissions: 显示该包的 manifest.permissions 详情
```

### J7: 切换 profile

```
Settings → Profiles
  → 显示当前激活 profile + 其它可用 profiles
  → 当前: default (forge-alpha)
  → 切换到另一个 profile 需重启 host
  → 显示 "切换需重启: [取消] [重启 host 切换到 alpha]"
  → 重启过程显示全屏 loading 然后回到 Home (新 profile 下)
```

### J8: 项目失败 / 崩溃

```
项目运行中突然 Failed (subprocess crash, timeout, error event)
  → 项目卡片状态变红 (Deep Rust)
  → Toast 滑入: "YdlTavern 已停止 (subprocess crash). [查看详情]"
  → 用户点详情 → 打开 Failure modal
       显示: 退出码 / stderr 末 50 行 (mono) / 时间戳
       Actions: [复制日志] [重启项目] [关闭]
  → 用户点重启 → kernel.v1.project.start (透明重启)
```

### J9: 命令面板 (Cmd+K, 后置)

```
任何路由按 Cmd+K (或 Ctrl+K)
  → 屏幕中央浮出搜索 modal, 背景虚化
  → 输入框 + 实时搜索结果列表
  → 搜索范围: 项目名, settings 项, 命令 (Install, Switch profile)
  → 上下箭头选择, 回车执行, Esc 退出
  → 类似 Linear / Raycast 命令面板
```

### J10: Theme 切换

```
topbar 右侧 sun/moon 图标 → 切换 light/dark
  → 即时生效 (CSS variable 切换)
  → localStorage 持久化, 启动时按用户偏好或 prefers-color-scheme
```

---

## 屏列表 (按设计优先级)

### 第一批 — 平台核心 (定调用, 必须先设计)

1. **Home (light + dark)** — 项目货架, asymmetric hero, 编辑感
2. **Home Empty** — 没装项目时的 first-run 状态
3. **Install Modal — URL 输入** — 流程入口
4. **Install Modal — 计划确认** — 显示包列表、权限、签名状态
5. **Install Modal — 进度** — 安装中
6. **External Project Wizard** — wrap / workspace 选择
7. **Project Frame topbar** — mounted iframe 的 chrome
8. **Toast / Notification** — 各种状态的 toast 样式

### 第二批 — Settings 三件套

9. **Settings — API Connections** — 平台密钥管理
10. **Settings — Installed Packages** — 包管理
11. **Settings — Profiles** — profile 切换

### 第三批 — Recovery & polish

12. **Failure Modal** — 项目崩溃详情
13. **Loading / Skeleton states** — 各处 skeleton 样式
14. **Settings — About** — 平台版本、许可、链接

### 第四批 — 后置 (有时间再做)

15. **Cmd+K Command Palette**
16. **Project detail (深度查看)**

---

## 组件目录 (复用)

按出现频率从高到低:

- **Project card** (Home 主元素)
- **Status pill** (Running / Stopped / Starting / Failed / Updating)
- **Primary button / Secondary button / Destructive button / Icon button**
- **Form input** (text / search / password / select / radio)
- **Toast** (info / success / error / warning, 但 warning 用 Aged Brass 不用黄)
- **Modal** (form / wizard / confirm)
- **Settings nav rail** (左侧二级导航)
- **Settings row** (label + control + helper text + divider)
- **Empty state** (composed icon + heading + body + optional CTA)
- **Error banner** (inline error 容器)
- **Skeleton loader** (针对 card / row / panel 不同形状)
- **Top bar** (platform topbar + project frame topbar 两种变体)
- **Drop menu** (⋯ 菜单, 用 Radix)
- **Tooltip** (hover 信息)
- **Tabs / Segmented control** (Settings 内可能用)
- **Progress bar** (Install 流程)

---

## 平台 / 项目边界

```
┌────────────────────────────────────────────┐
│  Platform Topbar (60px)                              │
│  Yggdrasil    /    Project: YdlTavern    [⚙] [🌗]    │
├────────────────────────────────────────────┤
│  Project Frame Topbar (40px)                         │
│  [←]  YdlTavern  • Running              [Stop] [⋯]  │
├────────────────────────────────────────────┤
│                                                       │
│                                                       │
│           Project iframe (free territory)             │
│           YdlTavern's own DOM lives here              │
│                                                       │
│                                                       │
└────────────────────────────────────────────┘
```

- 平台 topbar 始终可见 (sticky)
- 项目 frame topbar 仅在项目挂载时出现, 在平台 topbar 之下、iframe 之上
- iframe 内 YdlTavern (或任何项目) 完全自由, 平台不渗透
- 通信通过现有 postMessage RPC bridge (Round 10A 系列已建立)

---

## Stitch 生成顺序建议

Step 1 (本轮): Home (light), Home (dark), Home Empty, Install Modal URL 输入
  → 看图迭代风格. 改 DESIGN.md 直到满意

Step 2: 一致性确定后, 批量生成 Install 计划 / 进度 / Wizard / Project Topbar / Toast

Step 3: Settings 三件套

Step 4: Recovery + skeleton + 后置屏

每一步生成完, 由 @designer 接手做 React 实现, 不堆积一大批未实现的 figma 截图.

---

## 待用户决策的问题

(写到这里时还没问清, 需要在 Stitch 出图前/后确认)

1. **Topbar 双层还是单层?**
   - 双层 (60+40): 平台 topbar 始终在, 项目 topbar 项目挂载时叠加 (推荐)
   - 单层 (60): 项目挂载时平台 topbar 变形, 显示项目信息和 Stop
   - 推: 双层, 边界清楚

2. **First-run 有没有特殊欢迎屏?**
   - 是: 在装第一个项目前显示一个引导式欢迎页 (有 logo+slogan+CTA)
   - 否: 直接进 Home empty state, 用 empty state 引导
   - 推: 否 (Home empty state 已经能引导, 不堆 onboarding)

3. **是否做 marketing landing 页?**
   - 用户访问 yggdrasil.dev 这种官网入口? 还是只关注桌面端壳?
   - 推: 现在只做桌面端壳, marketing 页推后

4. **logo 字体?**
   - 用 Cabinet Grotesk 文字 logo (推荐, 简单直接)
   - 或 future: 设计一个简单 mark
   - 推: 文字 (本轮已确认)
