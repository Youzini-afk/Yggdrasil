# Provider Error Taxonomy

> 中文默认说明。本文件是 Model Provider Integration Alpha 的研究笔记。

Yggdrasil 不在 kernel 中定义模型错误。Provider adapters 应在 package output 与 diagnostics 中把上游错误映射到稳定分类，同时保留 provider 原始 code/message/request id 的 redacted metadata。

## 建议分类

- `bad_request`
- `authentication`
- `permission`
- `billing`
- `rate_limit`
- `not_found`
- `timeout`
- `overloaded`
- `tool_schema`
- `stream_error`
- `upstream_malformed`
- `network_denied`
- `secret_unavailable`
- `unknown`

建议附加字段：

- `retryable: boolean`
- `stage: preflight | request | stream | postprocess`
- `provider_family`
- `provider_code`
- `upstream_request_id`
- `redaction_state`

## 映射例子

- Anthropic `invalid_request_error`、Gemini `INVALID_ARGUMENT`、DeepSeek `400/422` → `bad_request`。
- OpenAI/Anthropic/OpenRouter/DeepSeek/xAI/Fireworks `401` → `authentication`。
- Anthropic/OpenRouter/DeepSeek/Fireworks `402` → `billing`。
- Gemini `RESOURCE_EXHAUSTED`、HTTP `429` → `rate_limit`。
- Anthropic `529 overloaded_error`、Gemini `UNAVAILABLE`、HTTP `502/503` → `overloaded`。
- Anthropic `timeout_error`、Gemini `DEADLINE_EXCEEDED`、HTTP `408/504` → `timeout`。
- Strict schema/tool argument validation failures → `tool_schema`。
- SSE error event or malformed stream chunk → `stream_error` / `upstream_malformed`。

## 非目标

- 不做用户余额或账单系统。
- 不在 kernel 中新增 provider error enum。
- 不持久化 raw request/response body。
