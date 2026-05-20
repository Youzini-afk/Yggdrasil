# Conformance Feedback Loop 使用指南

> [English](./CONFORMANCE_FEEDBACK.en.md) · [中文](./CONFORMANCE_FEEDBACK.md)

`ygg conformance` 命令支持过滤、计时和诊断，帮助快速定位失败和慢 case。

## 基本用法

```bash
# 运行全部 260 个 conformance cases（默认行为不变）
cargo run -p ygg-cli -- conformance
```

## 列出 cases

```bash
# 列出所有 case id 和 tags，不执行
cargo run -p ygg-cli -- conformance --list
```

输出格式：`<case_id>  [<tag1>, <tag2>, ...]`

## 按 case id 过滤

```bash
# 按 substring 过滤（匹配 case id 中包含指定子串的 cases）
cargo run -p ygg-cli -- conformance --case sharing_lab.contract_shape

# 过滤所有 sharing_lab 的 cases
cargo run -p ygg-cli -- conformance --case sharing_lab
```

## 按 tag 过滤

```bash
# 按 tag 过滤（case 只需包含任一指定 tag 即被选中）
cargo run -p ygg-cli -- conformance --tag sharing

# 组合多个 tag（OR 语义）
cargo run -p ygg-cli -- conformance --tag network --tag secret
```

## Fail-fast

```bash
# 第一个失败后立即停止
cargo run -p ygg-cli -- conformance --fail-fast
```

## Slowest report

```bash
# 末尾显示最慢 10 个 cases（默认）
cargo run -p ygg-cli -- conformance

# 自定义最慢 N
cargo run -p ygg-cli -- conformance --slowest 3
```

## 组合使用

```bash
# 只跑 sharing 相关 cases，fail-fast，显示最慢 3
cargo run -p ygg-cli -- conformance --tag sharing --fail-fast --slowest 3

# 按 case id 和 tag 同时过滤（AND 语义：必须同时满足两个条件）
cargo run -p ygg-cli -- conformance --case sharing_lab --tag secret
```

## 可用 tags

| Tag | 说明 |
|---|---|
| runtime | 内核 runtime 行为（session、event、capability、hook 等） |
| session | Session 生命周期 |
| event | 事件追加与读取 |
| capability | 能力发现与调用 |
| package | 包加载、卸载、restart |
| official | 官方包 conformance |
| schema | JSON Schema 验证 |
| protocol | 公开协议分发 |
| permission | 权限/principal |
| hook | Hook fabric 切片 |
| subprocess | Subprocess 包执行（通常较慢） |
| host | Host 诊断与 profile |
| surface | Surface contribution |
| proposal | Proposal lifecycle |
| asset | Asset 注册表 |
| projection | Projection 注册表 |
| substrate | SQLite 底座 |
| composition | Composition descriptor |
| replacement | 第三方替换证明 |
| generated | 生成的包模板 conformance（通常较慢） |
| secret | Secret reference / raw-secret blocking |
| network | 网络权限与出站 |
| outbound | 出站 executor boundary |
| live | Live model calls |
| stream | Streaming 生命周期 |
| agentic | Agentic Forge |
| experience | Experience runtime / playable board |
| memory | Memory lab |
| sharing | Sharing lab |
| slow | 已知较慢的 cases（subprocess 启动、生成包模板等） |

## 输出格式

每个 case 输出一行：

```
<case_id>  PASS|FAIL  <duration>
```

末尾汇总 slowest N 和总体结果。
