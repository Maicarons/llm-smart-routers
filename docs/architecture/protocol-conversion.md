# 协议转换引擎

## 概述

协议转换引擎负责在 OpenAI Chat Completions、OpenAI Responses、Anthropic Messages 三种格式之间进行无损双向转换。

## 支持的协议

| 端点 | 协议 | 请求格式 | 响应格式 | 流式格式 |
|------|------|---------|---------|---------|
| `/v1/chat/completions` | OpenAI | ChatCompletionRequest | ChatCompletionResponse | `choices[*].delta` |
| `/v1/messages` | Anthropic Messages | MessagesRequest | MessagesResponse | `content_block_delta` |
| `/v1/responses` | OpenAI Responses | ResponsesRequest | ResponsesResponse | 基于事件的 SSE |

## 转换架构

```
外部请求
  │
  ├─→ 协议检测 (基于 URL 路径)
  │
  ├─→ 解析为 Canonical Model
  │    ├─ UnifiedRequest
  │    ├─ UnifiedMessage
  │    ├─ UnifiedContent
  │    └─ UnifiedResponse
  │
  ├─→ 路由决策 (基于 Canonical Model)
  │
  ├─→ 转换为提供商格式
  │    ├─ OpenAI ChatCompletionRequest
  │    └─ Anthropic MessagesRequest
  │
  └─→ 调用提供商 API
```

## 核心转换映射

| OpenAI Chat Completions | OpenAI Responses | Canonical | Anthropic Messages |
|------------------------|-----------------|-----------|-------------------|
| `messages[0].role="system"` → `system` | `instructions` | `system_prompt` | `system` 顶层字段 |
| `messages[0].content` | `input` | `content` | `messages[0].content` |
| `max_tokens` / `max_completion_tokens` | `max_output_tokens` | `max_tokens` | `max_tokens` |
| `tools` | `tools` | `tools` | `tools` |
| `stream: true` | `stream: true` | `stream` | `stream: true` |
| `stop: ["\n\n"]` | `stop: ["\n\n"]` | `stop_sequences` | `stop_sequences: ["\n\n"]` |