# 配置文件

## default.yaml

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4

storage:
  duckdb:
    path: "data/smart_router.duckdb"
    auto_archive: true
    archive_threshold_days: 30
  cache:
    ttl_default_sec: 300
    max_capacity: 10000
  json_config:
    path: "data/config/"
    auto_reload: true

auth:
  api_keys:
    - key: "sk-your-key-1"
      name: "default"

router:
  default_strategy: "task_aware"
  strategies:
    - name: "standard"
      type: "task_aware"
      scoring:
        capability_weight: 0.4
        price_weight: 0.2
        latency_weight: 0.2
        quality_weight: 0.15
        health_weight: 0.05
```

## 提供商配置

```json
{
  "providers": [
    {
      "name": "my-openai",
      "api_base_url": "https://api.openai.com/v1",
      "api_key": "sk-...",
      "models": [
        { "id": "gpt-4o", "capabilities": ["chat", "vision"] },
        { "id": "gpt-4o-mini", "capabilities": ["chat"] }
      ]
    }
  ]
}
```