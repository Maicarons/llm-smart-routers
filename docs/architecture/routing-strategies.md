# 路由策略引擎

## 架构

```
用户请求 → 策略选择器 → 策略执行链 → 模型选择结果
                │              │
                ▼              ▼
        策略仓库          中间件管道
        · 手动指定         · 任务分类器
        · 故障转移         · 熔断检查器
        · 负载均衡         · 健康评分器
        · 成本优先         · 模型排序器
        · 任务感知
        · 评分优化
        · 自适应
```

## 评分公式

```
Score(model, context) = W₁ × Capability(task, model)
                      + W₂ × (1 - normalized_price)
                      + W₃ × (1 - normalized_latency)
                      + W₄ × quality_score
                      + W₅ × health_score
```

## 任务分类体系

| 任务类型 | 说明 |
|---------|------|
| code_generation | 代码生成 |
| translation | 翻译/本地化 |
| creative_writing | 创意写作 |
| analysis | 分析推理/数学 |
| summarization | 摘要/信息提取 |
| general_qa | 问答/知识检索 |
| tool_use | 工具调用 |
| brainstorming | 头脑风暴 |