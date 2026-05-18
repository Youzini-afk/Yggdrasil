# Stream Compatibility Notes

> 中文默认说明。本文件是 Model Provider Integration Alpha 的研究笔记。

不要把所有模型流都抽象成 OpenAI Chat Completions 的 `choices[].delta`。Yggdrasil provider adapter 应把不同上游流解析成 package-owned normalized stream events，再由 host streaming substrate 包装为 content-free `StreamFrameEnvelope`。

## 主要流族

### Delta SSE

代表：OpenAI Chat Completions、OpenAI-compatible、DeepSeek、xAI Chat、Fireworks Chat、OpenRouter Chat。

常见特征：

- `data: {...}` SSE lines。
- `data: [DONE]` terminal marker。
- 文本在 `choices[].delta.content`。
- tool call arguments 可能分片到多个 delta。
- usage 可能只在最终 chunk 或非 streaming response 中出现。

### Semantic SSE

代表：OpenAI Responses、Anthropic Messages、OpenRouter Responses。

常见特征：

- 上游事件有显式语义，例如 `response.output_text.delta`、`message_start`、`content_block_delta`、`message_delta`。
- tool call start/args/done 不是简单文本 delta。
- error 可能作为 stream event 出现。

### Typed chunk stream

代表：Gemini `streamGenerateContent`。

常见特征：

- chunk 是 `GenerateContentResponse` 增量对象。
- 文本在 `candidates[].content.parts[].text`。
- finish reason、safety/block reason、usage metadata 需要单独映射。

## Ygg provider adapter normalized events

建议 SDK/package 层使用：

- `text_delta`
- `reasoning_delta`
- `tool_call_started`
- `tool_args_delta`
- `tool_call_done`
- `citation`
- `usage_final`
- `error`
- `done`
- `heartbeat`

这些是 provider package output/trace 语义，不是 kernel 语义。Kernel 仍只处理 content-free stream frames。

## 必须覆盖的边界

- Mid-stream provider error。
- Missing `[DONE]` 或 terminal event。
- Heartbeat / keepalive comments。
- Tool argument JSON fragment。
- Usage only in final chunk。
- Client cancel 后不能继续 append chunk。
- Timeout 后必须 terminal。
- Redaction state per frame。
