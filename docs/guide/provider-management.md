# 提供商管理

## 添加提供商

通过 REST API 注册：

```bash
curl -X POST http://localhost:8080/admin/providers \
  -H "Authorization: Bearer admin-key" \
  -d '{
    "name": "my-openai",
    "api_base_url": "https://api.openai.com/v1",
    "api_key": "sk-...",
    "models": [
      {"id": "gpt-4o", "capabilities": ["chat", "vision"]},
      {"id": "gpt-4o-mini", "capabilities": ["chat"]}
    ]
  }'
```

或通过 JSON 配置文件：

```json
{
  "providers": [
    {
      "name": "my-openai",
      "api_base_url": "https://api.openai.com/v1",
      "api_key": "sk-...",
      "models": [
        { "id": "gpt-4o", "capabilities": ["chat", "vision"] }
      ]
    }
  ]
}
```

## 支持的提供商

系统不内置任何提供商，但适配器层支持以下 API 格式：

| 提供商 | API 格式 | 适配器 |
|--------|---------|--------|
| OpenAI | Chat Completions | OpenAIAdapter |
| Anthropic | Messages / Responses | AnthropicAdapter |
| Google AI | Gemini | GoogleAdapter |
| Azure OpenAI | Chat Completions | AzureAdapter |
| 兼容 OpenAI 的 API | Chat Completions | OpenAIAdapter |