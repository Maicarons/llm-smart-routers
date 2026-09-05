# API 参考

## 端点总览

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/v1/chat/completions` | OpenAI 兼容接口 |
| POST | `/v1/messages` | Anthropic Messages 兼容接口 |
| POST | `/v1/responses` | OpenAI Responses 兼容接口 |
| GET | `/v1/models` | 列出可用模型 |
| GET | `/health` | 健康检查 |
| POST | `/admin/providers` | 注册提供商 |
| GET | `/admin/providers` | 列出提供商 |
| DELETE | `/admin/providers/:name` | 删除提供商 |

## 统一调用示例

### OpenAI 格式

```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-your-key" \
  -d '{
    "model": "auto",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### Anthropic 格式

```bash
curl http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-your-key" \
  -d '{
    "model": "auto",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 1000
  }'
```

### OpenAI Responses 格式

```bash
curl http://localhost:8080/v1/responses \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-your-key" \
  -d '{
    "model": "auto",
    "input": "Hello"
  }'
```