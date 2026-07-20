# Contract Registry 与显式协商（Experimental）

> [English](./CONTRACT_REGISTRY.en.md) · [中文](./CONTRACT_REGISTRY.md)

本文描述分层合同迁移的第一套可执行兼容机制。它是 Experimental，不表示
Constitution v2 已经 Stable，也不改变现有 `kernel.v1.*` payload 语义。

## 单一解析边界

Runtime 在进入 permission gate 和 handler 前统一执行：

1. 校验可选的 contract selection；
2. 从集中 registry 解析 canonical ID 或 alias；
3. 执行 request adapter；
4. 以 `KernelMethod` 调用唯一 handler；
5. 执行 response adapter。

HTTP RPC、host stdio、in-process 和 subprocess reverse stdio 使用同一解析与协商逻辑。
Alias 不创建新 handler、principal、permission 或 audit 分支。

## Registry 形状

每个 `ContractMethod` 发布：

- `canonical_id` 与 `aliases`；
- `owner_layer` 与 `maturity`；
- request / response schema URI；
- request / response adapter；
- introduced / deprecated / replacement metadata；
- 当前实现状态与 streaming 标记。

当前第一批 identity alias 为：

| Canonical | Legacy alias | Owner |
|---|---|---|
| `host.info` | `kernel.v1.host.info` | `host` |
| `host.target.list` | `kernel.v1.target.list` | `host` |

其他方法在迁移前继续以现有 `kernel.v1.*` ID 作为 canonical ID。新增 alias 必须进入
registry，不能在 dispatcher、客户端或 transport 中加入字符串特判。

## 显式协商

RPC envelope 可带可选字段：

```json
{
  "id": "request-1",
  "method": "host.info",
  "params": {},
  "contract": {
    "profile": "ygg.contract.default/v1",
    "versions": [
      { "layer": "host", "version": "0.1.0" }
    ]
  }
}
```

- 省略 `contract` 时，为旧客户端使用 `kernel.v1` legacy profile。
- 一旦客户端显式给出 profile 或 layer version，host 必须精确满足。
- 未知 profile、profile 不包含所需 layer、或 version 不匹配时返回
  `kernel/v1/error/unsupported_contract`，并在结构化 details 中报告原因。
- Host 不会自动回退到更弱 profile，也不会在协商失败后调用业务 handler。

## `host.info`

原有 `protocol_version`、`methods`、`supported_transports` 保持不变。新增字段均为
additive optional 字段：

- `contract_registry_version`、`default_profile`；
- `layers`、`versions`、`profiles`、`maturity`；
- `aliases`、`contract_methods`。

因此旧 SDK 可以忽略新字段；新 SDK 连接旧 host 时也必须允许这些字段缺失。

## SDK

生成器读取每个 method schema 的 `x-yggdrasil-contract` metadata：

- 原有方法名调用 canonical wire ID；
- 每个 legacy wire ID 生成显式 `legacyKernelV1...` / `legacy_kernel_v1_...`
  wrapper；
- negotiated client 只有在 transport 能携带 contract selection 时才启用，不能静默忽略选择。

Schema、SDK 与 OpenAPI 必须通过生成器更新，不手工修改生成物。

