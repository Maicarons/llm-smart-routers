# Phase 1: 核心引擎 (MVP)

## 目标

构建最小可行产品：统一 API 接口、基础路由、提供商管理、简易熔断。预计 8-10 周。

## 里程碑

### M1: 项目骨架 (第 1 周)

- Cargo workspace 配置
- CI/CD 流水线
- 文档框架搭建
- 代码规范与 lint 配置

### M2: 统一 API 接口 (第 2-3 周)

- 三大协议端点的请求/响应格式定义
- 协议转换核心 (OpenAI ↔ Canonical ↔ Anthropic)
- 流式 SSE 事件转换
- 基础 Axum 服务端

### M3: 提供商适配器 (第 4-5 周)

- ProviderAdapter trait 定义
- OpenAI 适配器实现
- Anthropic 适配器实现
- 提供商注册中心 (ProviderRegistry)
- 用户自注册 REST API

### M4: 基础熔断器 (第 6 周)

- 熔断状态机 (Closed/Warning/Degraded/Limited/Open)
- 滑动窗口实时统计
- 健康度评分引擎
- 熔断器与路由引擎的集成

### M5: 基础路由策略 (第 7 周)

- Strategy trait 定义
- 手动指定策略
- 故障转移策略
- 负载均衡策略

### M6: 存储 & 配置 (第 8 周)

- DuckDB 嵌入集成
- moka + dashmap 缓存层
- JSON 配置热重载
- 默认配置文件

### M7: 集成测试 (第 9-10 周)

- 端到端路由测试
- 协议转换测试
- 熔断器行为测试
- 性能基准测试

## 交付标准

- [ ] `cargo build --release` 通过
- [ ] `cargo test --workspace` 全部通过
- [ ] 三大协议端点均可调用
- [ ] 用户可通过 JSON 文件注册提供商
- [ ] 故障转移路由正常工作
- [ ] 基础熔断器在异常时切换模型
- [ ] 调用日志写入 DuckDB