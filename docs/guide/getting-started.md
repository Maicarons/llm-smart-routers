# 快速开始

## 前置要求

- Rust 1.80+
- 无任何外部服务依赖

## 安装

```bash
# 克隆项目
git clone https://github.com/your-org/llm-smart-routers
cd llm-smart-routers

# 构建
cargo build --release

# 运行
./target/release/llm-smart-router-gateway
```

## 配置

创建 `data/config/providers.json`：

```json
{
  "providers": [
    {
      "name": "my-openai",
      "api_base_url": "https://api.openai.com/v1",
      "api_key": "sk-your-key-here",
      "models": [
        { "id": "gpt-4o", "capabilities": ["chat", "vision"] },
        { "id": "gpt-4o-mini", "capabilities": ["chat"] }
      ]
    }
  ]
}
```

## 使用

```bash
# OpenAI 兼容接口
curl http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer sk-your-key" \
  -d '{"model": "auto", "messages": [{"role": "user", "content": "Hello"}]}'

# Anthropic 兼容接口
curl http://localhost:8080/v1/messages \
  -H "x-api-key: sk-your-key" \
  -d '{"model": "auto", "messages": [{"role": "user", "content": "Hello"}], "max_tokens": 100}'
```