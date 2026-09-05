# LLM Smart Router

> **次世代智能 AI 路由引擎** — 高性能、多协议统一、智能熔断，实时为用户选择最优模型。

[![CI](https://github.com/Maicarons/llm-smart-routers/actions/workflows/ci.yml/badge.svg)](https://github.com/Maicarons/llm-smart-routers/actions/workflows/ci.yml)
[![Docs](https://github.com/Maicarons/llm-smart-routers/actions/workflows/docs.yml/badge.svg)](https://maicarons.github.io/llm-smart-routers/)
[![License](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)

---

## 目录

- [核心设计理念](#核心设计理念)
- [技术栈](#技术栈)
- [多协议统一架构](#多协议统一架构)
- [智能熔断系统](#智能熔断系统)
- [路由策略引擎](#路由策略引擎)
- [系统架构](#系统架构)
- [项目结构](#项目结构)
- [开发路线图](#开发路线图)
- [快速开始](#快速开始)
- [数据模型](#数据模型)
- [FAQ](#faq)

---

## 核心设计理念

### 设计原则

| 原则 | 说明 |
|------|------|
| **零内置提供商** | 用户完全掌控自己的 API 接口信息，系统只做路由和管理 |
| **协议统一层** | 兼容 OpenAI Chat Completions、OpenAI Responses、Anthropic Messages 三大协议 |
| **后端优先** | 核心引擎是独立后台服务，WebUI 是分离的可选组件 |
| **智能熔断优先** | 熔断不是事后补救，而是预测性、渐进式的流量控制 |
| **性能至上** | Rust 核心，零拷贝，无 GC 抖动，亚毫秒级路由决策 |

### 与传统 API 网关的本质区别

```
传统 API 网关                    LLM Smart Router
─────────────────                ─────────────────
只做请求转发                      + 任务感知 + 智能选择
静态路由规则                      + 动态评分 + 自适应学习
简单熔断 (开/关)                 + 预测性 + 渐进式熔断
单一协议代理                      + 多协议统一翻译
内置提供商耦合                    + 用户自管理接口
```

---

## 技术栈

### 选型

| 技术 | 用途 | 选择理由 |
|------|------|---------|
| **Rust** | 编程语言 | 零成本抽象、无 GC、内存安全、静态编译单二进制 |
| **Axum** | Web 框架 | 基于 Tower 中间件栈，生态集成好，支持提取器模式 |
| **Tokio** | 异步运行时 | Rust 事实标准，work-stealing 调度器，I/O 驱动 |
| **Tower** | 中间件层 | 可组合的 Service 抽象，天然支持链路追踪、限流、熔断 |
| **duckdb** (crate) | 嵌入式分析数据库 | 列式存储、单文件、ACID、SQL 全支持，日志分析比 PG 快 10-50 倍 |
| **moka** | 并发缓存 | TTL/TTI 支持、锁无关分片架构、async 原生，Java Caffeine 移植 |
| **dashmap** | 并发 HashMap | 无锁分片，读比 `RwLock<HashMap>` 快 10-100 倍，适合熔断/限流 |
| **serde_json** | 配置序列化 | JSON 配置管理，人类可读，支持 Git 版本管理和热重载 |

### 设计原则

- **零外部依赖**：所有数据层（缓存、状态、日志、配置）均在进程内管理
- **单二进制部署**：`cargo build` 后只有一个可执行文件，无运行时依赖
- **嵌入式存储**：DuckDB（分析）+ moka/dashmap（内存）+ JSON 文件（配置）

---

## 多协议统一架构

### 支持的协议端点

```
POST /v1/chat/completions     → OpenAI Chat Completions 格式
POST /v1/messages             → Anthropic Messages 格式
POST /v1/responses            → OpenAI Responses 格式
POST /v1/embeddings           → 统一 Embeddings 接口
POST /v1/models               → 列出可用模型
```

### 协议转换引擎

核心挑战：不同 API 的请求/响应格式差异巨大，需要无损双向转换。

```
                        ┌──────────────────────┐
                        │   协议检测 & 路由      │
                        │   (基于 URL 路径)      │
                        └──────────┬───────────┘
                                   │
                                   ▼
┌──────────────────────────────────────────────────────────┐
│                    协议转换适配器                          │
│                                                          │
│  ┌──────────────────┐  ┌──────────────────────────────┐  │
│  │  OpenAI 格式      │  │  Anthropic 格式              │  │
│  │  ────────────    │  │  ────────────                │  │
│  │  model: "auto"   │  │  model: "claude-3-5-sonnet"  │  │
│  │  messages: [     │  │  messages: [                 │  │
│  │    {role,content} │  │    {role,content}            │  │
│  │  ]                │  │  ]                           │  │
│  │  max_tokens       │  │  max_tokens                  │  │
│  │  temperature      │  │  temperature                 │  │
│  │  tools            │  │  tools                       │  │
│  │  stream: true     │  │  stream: true                │  │
│  └──────────────────┘  └──────────────────────────────┘  │
│                                                          │
│  ┌──────────────────────────────────────────────────┐    │
│  │          内部统一模型 (Canonical Model)            │    │
│  │  ┌────────────────────────────────────────────┐   │    │
│  │  │  UnifiedRequest {                          │   │    │
│  │  │    model: String,                          │   │    │
│  │  │    messages: Vec<UnifiedMessage>,           │   │    │
│  │  │    system_prompt: Option<String>,           │   │    │
│  │  │    max_tokens: Option<u32>,                 │   │    │
│  │  │    temperature: Option<f32>,                │   │    │
│  │  │    tools: Vec<UnifiedTool>,                 │   │    │
│  │  │    stream: bool,                           │   │    │
│  │  │    metadata: HashMap<String,String>,         │   │    │
│  │  │  }                                          │   │    │
│  │  └────────────────────────────────────────────┘   │    │
│  └──────────────────────────────────────────────────┘    │
│                                                          │
│  ┌──────────────────┐  ┌──────────────────────────────┐  │
│  │  OpenAI 响应      │  │  Anthropic 响应              │  │
│  │  ────────────    │  │  ────────────                │  │
│  │  choices[0].     │  │  content[0].text             │  │
│  │  message.content │  │  stop_reason: "end_turn"     │  │
│  │  finish_reason   │  │  usage.{input,output}_tokens │  │
│  │  usage           │  │  (流式: content_block_start)  │  │
│  └──────────────────┘  └──────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### 关键转换映射

#### Request 转换

| OpenAI Chat Completions | OpenAI Responses | Anthropic Messages | 说明 |
|------------------------|-----------------|-------------------|------|
| `model` | `model` | `model` | 直接映射 |
| `messages[0].role="system"` → `system` | `instructions` | `system` | Anthropic 顶层字段 |
| `messages[0].content` | `input` | `messages[0].content` | 字符串或内容块数组 |
| `messages[role="tool"]` | `input` | `messages[role="user"]` + `content: [tool_result]` | 工具结果格式不同 |
| `max_tokens` / `max_completion_tokens` | `max_output_tokens` | `max_tokens` | 统一为 `max_tokens` |
| `tools` | `tools` | `tools` | 格式需要转换 |
| `stream: true` | `stream: true` | `stream: true` | 流式，但事件格式不同 |
| `stop: ["\n\n"]` | `stop: ["\n\n"]` | `stop_sequences: ["\n\n"]` | 字段名不同 |

#### Response 转换

| OpenAI Chat Completions | OpenAI Responses | Anthropic Messages | 说明 |
|------------------------|-----------------|-------------------|------|
| `choices[0].message.content` | `output[0].content[0].text` | `content[0].text` | 文本内容 |
| `choices[0].finish_reason: "stop"` | `status: "completed"` | `stop_reason: "end_turn"` | 正常结束 |
| `choices[0].finish_reason: "length"` | `status: "incomplete"` | `stop_reason: "max_tokens"` | 长度截断 |
| `choices[0].finish_reason: "tool_calls"` | `output[0].type: "tool_call"` | `stop_reason: "tool_use"` | 工具调用 |
| `choices[0].finish_reason: "content_filter"` | `status: "incomplete"` | `stop_reason: "end_turn"` | 内容过滤 |
| `usage.prompt_tokens` | `usage.input_tokens` | `usage.input_tokens` | 输入 Token |
| `usage.completion_tokens` | `usage.output_tokens` | `usage.output_tokens` | 输出 Token |

#### 流式 (SSE) 转换

| OpenAI Delta | OpenAI Responses 事件 | Anthropic 事件 | 说明 |
|-------------|----------------------|---------------|------|
| `choices[0].delta.content` | `output` 项中 `type: "text"` 的增量 | `content_block_delta` | 文本增量 |
| `choices[0].delta.tool_calls` | `output` 项中 `type: "tool_call"` 的增量 | `content_block_start` (type: tool_use) | 工具调用开始 |
| `choices[0].finish_reason` | `status: "completed"` 信号 | `message_delta` → `delta.stop_reason` | 结束信号 |
| `usage` | `usage` 字段 | `message_delta` → `usage` | 最终用量 |

### 协议检测策略

```
请求进入 → 检查 URL 路径
  ├── /v1/chat/completions → OpenAI Chat Completions 解析
  ├── /v1/messages         → Anthropic Messages 解析
  ├── /v1/responses        → OpenAI Responses 解析
  └── /v1/*                → 根据 Accept 头或请求体自动检测
```

---

## 智能熔断系统

### 行业现状分析

现有熔断器方案的局限性：

| 方案 | 问题 |
|------|------|
| **Netflix Hystrix** (2012) | 固定阈值，不支持自适应，已进入维护模式 |
| **Resilience4j** | 基于滑动窗口的计数，阈值为静态配置，无法提前预测 |
| **Polly (.NET)** | 与 Hystrix 类似的基于计数/时间窗口，无预测能力 |
| **Sentinel (Alibaba)** | 支持实时统计和自适应，但主要面向 Java 微服务场景 |
| **Google SRE 熔断** | 基于客户端请求预算，不感知服务端实际状态 |
| **AWS 熔断** | 基于 HTTP 503 错误率，粒度较粗 |

### 本项目的创新：五维智能熔断引擎

我们设计了一套**五维智能熔断系统**，在传统熔断器的基础上引入预测性、多维度、渐进式熔断机制。

```
┌──────────────────────────────────────────────────────────────────┐
│                    五维智能熔断引擎                                │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  维度 1: 实时故障统计 (传统基础)                          │    │
│  │  ├── 滑动窗口错误率 (5s/30s/120s 三个窗口)               │    │
│  │  ├── 错误类型分类 (timeout, rate_limit, server_error,    │    │
│  │  │                   auth_error, invalid_response)        │    │
│  │  └── 各类型独立阈值，避免单一错误类型过早熔断             │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  维度 2: 延迟趋势预测 (提前发现)                          │    │
│  │  ├── 指数加权移动平均 (EWMA) 追踪 P50/P95/P99 延迟趋势   │    │
│  │  ├── 延迟加速度检测：连续 3 个窗口延迟递增 → 预警        │    │
│  │  ├── 季节性延迟模式识别：识别周期性延迟峰值               │    │
│  │  └── 提前进入"半预警"状态，降低流量而非直接断开           │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  维度 3: 健康度评分 (渐进式)                              │    │
│  │  ├── 综合评分: 0.0 (完全不可用) ~ 1.0 (完全健康)         │    │
│  │  ├── 评分公式:                                            │    │
│  │  │  Health = W_err × (1 - error_rate)                     │    │
│  │  │          + W_lat × latency_score                       │    │
│  │  │          + W_recent × recent_trend_score               │    │
│  │  │          + W_slo × slo_compliance_score                │    │
│  │  ├── 渐进式路由降级:                                       │    │
│  │  │  Health < 0.8 → 降低 20% 流量                          │    │
│  │  │  Health < 0.6 → 降低 50% 流量                          │    │
│  │  │  Health < 0.3 → 降低 80% 流量                          │    │
│  │  │  Health < 0.1 → 完全熔断                                │    │
│  │  └── 取代传统二值 (Open/Closed) 熔断                      │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  维度 4: 主动健康探测 (Proactive Probing)                 │    │
│  │  ├── 定期发送轻量探测请求 (如 "hello" 或简单 embedding)   │    │
│  │  ├── 探测间隔自适应：健康时 30s/次，不稳定时 5s/次       │    │
│  │  ├── 探测结果独立于业务流量，避免偏置                      │    │
│  │  └── 快速恢复：探测成功 2 次 → 健康度 +0.2               │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  维度 5: 提供商级联熔断 (Provider Cascading)             │    │
│  │  ├── 同一提供商下多个模型共享健康状态                    │    │
│  │  ├── 模型级熔断 → 提供商级预警 → 区域级熔断             │    │
│  │  ├── 示例：us-east-1 区域两个模型同时延迟升高 →          │    │
│  │  │       区域级健康度降低 → 自动切换 eu-west-1           │    │
│  │  └── 级联传播：单一模型故障不扩散到整个提供商            │    │
│  └──────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

### 熔断状态机

```
                    ┌──────────────┐
                    │              │
        初始状态 ──→│  全流量 (1.0) │←──── 恢复成功
                    │              │
                    └──────┬───────┘
                           │ 延迟趋势预警
                           ▼
                    ┌──────────────┐
                    │              │
                    │  预警 (0.8)  │──→ 降低 20% 流量
                    │              │
                    └──────┬───────┘
                           │ 健康度持续下降
                           ▼
                    ┌──────────────┐
                    │              │
                    │  降级 (0.6)  │──→ 降低 50% 流量
                    │              │
                    └──────┬───────┘
                           │ 健康度恶化
                           ▼
                    ┌──────────────┐
                    │              │
                    │  限制 (0.3)  │──→ 降低 80% 流量
                    │              │
                    └──────┬───────┘
                           │ 完全不可用
                           ▼
                    ┌──────────────┐
                    │              │
                    │  熔断 (0.0)  │──→ 100% 流量切换到备用
                    │              │      主动探测恢复
                    └──────┬───────┘
                           │ 探测成功
                           ▼
                    ┌──────────────┐
                    │              │
                    │  半开 (0.5)  │──→ 逐步放量 10%→30%→60%
                    │              │
                    └──────┬───────┘
                           │ 验证通过
                           ▼
                    ┌──────────────┐
                    │              │
                    │  全流量 (1.0) │
                    │              │
                    └──────────────┘
```

### 熔断配置示例

```yaml
# config/circuit_breaker.yaml
circuit_breaker:
  # 滑动窗口
  sliding_window:
    size_seconds: 120
    bucket_count: 12  # 每个 bucket 10s

  # 错误类型阈值
  error_thresholds:
    timeout:
      trigger: 0.15   # 15% 超时率触发
      severity: 0.4   # 权重
    rate_limit:
      trigger: 0.10
      severity: 0.3
    server_error:
      trigger: 0.05
      severity: 0.5
    auth_error:
      trigger: 0.01
      severity: 0.8

  # 延迟趋势
  latency_trend:
    ewma_alpha: 0.3           # 平滑系数
    acceleration_windows: 3   # 连续递增窗口数
    p95_threshold_ms: 5000    # P95 超过 5s 预警

  # 健康度评分权重
  scoring:
    error_rate_weight: 0.4
    latency_weight: 0.3
    trend_weight: 0.2
    slo_weight: 0.1

  # 主动探测
  probing:
    enabled: true
    interval_healthy_sec: 30
    interval_unstable_sec: 5
    probe_timeout_ms: 3000
    recovery_success_count: 2
```

---

## 路由策略引擎

### 策略架构

```
┌────────────────────────────────────────────────────────────┐
│                    路由策略引擎                              │
│                                                            │
│  用户请求 ──→ 策略选择器 ──→ 策略执行链 ──→ 模型选择结果    │
│                   │               │                         │
│                   ▼               ▼                         │
│           ┌──────────────┐  ┌───────────────────┐          │
│           │ 策略仓库      │  │ 中间件管道         │          │
│           │              │  │ ┌───────────────┐ │          │
│           │ ▪ 手动指定   │  │ │ 任务分类器     │ │          │
│           │ ▪ 故障转移   │  │ ├───────────────┤ │          │
│           │ ▪ 负载均衡   │  │ │ 熔断检查器     │ │          │
│           │ ▪ 成本优先   │  │ ├───────────────┤ │          │
│           │ ▪ 任务感知   │  │ │ 健康评分器     │ │          │
│           │ ▪ 评分优化   │  │ ├───────────────┤ │          │
│           │ ▪ 自适应     │  │ │ 模型排序器     │ │          │
│           │ ▪ 自定义插件  │  │ └───────────────┘ │          │
│           └──────────────┘  └───────────────────┘          │
└────────────────────────────────────────────────────────────┘
```

### 策略详解

#### 1. 手动指定策略

```
用户指定 model = "gpt-4o" → 直接路由到 gpt-4o
失败时 → 根据 fallback 列表依次尝试
```

#### 2. 故障转移策略

```
主模型 (gpt-4o) ──→ 成功? ──→ 返回
                  └── 失败 ──→ 备用1 (claude-sonnet) ──→ 成功? ──→ 返回
                                          └── 失败 ──→ 备用2 (gemini-pro) ──→ ...
```

#### 3. 负载均衡策略

| 算法 | 描述 | 适用场景 |
|------|------|---------|
| Round Robin | 依次轮询 | 同模型多 Key 均衡 |
| Weighted | 按权重分配 | 不同速率限制的 Key |
| Least Load | 选当前负载最低 | 异构部署环境 |
| Consistent Hash | 相同请求落到同一模型 | 缓存友好 |

#### 4. 成本优先策略

```
1. 确定任务最低能力要求 (如: 翻译任务需要 ≥ 7.0 分)
2. 筛选满足要求的模型
3. 按每百万 token 价格升序排列
4. 选择价格最低的可用模型
5. 检查健康度 ≥ 0.3 → 使用，否则选下一个
```

#### 5. 任务感知评分策略 (核心智能策略)

```
Score(model, context) = W₁ × Capability(task, model)
                      + W₂ × (1 - normalized_price)
                      + W₃ × (1 - normalized_latency)
                      + W₄ × quality_score
                      + W₅ × health_score

任务分类体系:
  ├── code_generation    (代码生成)
  ├── code_explanation   (代码解释)
  ├── translation        (翻译/本地化)
  ├── creative_writing   (创意写作)
  ├── analysis           (分析推理/数学)
  ├── summarization      (摘要/信息提取)
  ├── general_qa         (问答/知识检索)
  ├── tool_use           (工具调用/函数调用)
  └── brainstorming      (头脑风暴)

```

#### 6. 自适应优化策略

```
┌─────────────┐    ┌──────────────┐    ┌──────────────┐
│  收集调用数据  │──→│  更新模型画像  │──→│  优化路由权重  │
│              │    │              │    │              │
│  ✔ 响应时间   │    │  更新延迟画像  │    │  提升高质量模型  │
│  ✔ Token消耗  │    │  更新成功率    │    │  降低低质量模型  │
│  ✔ 是否成功   │    │  更新质量评分  │    │  重新评分排序  │
│  ✔ 用户反馈   │    │  更新成本数据  │    └──────────────┘
│  ✔ 输出质量   │    └──────────────┘
└─────────────┘
```

---

## 系统架构

### 高层架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│  外部客户端                                                         │
│  (OpenAI SDK / Anthropic SDK / HTTP Client / LangChain / Vercel AI) │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    API 网关层 (Gateway Service)                      │
│                                                                      │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌──────────────────┐ │
│  │ TLS 终止   │  │ 认证鉴权   │  │ 速率限制   │  │ 请求日志/追踪    │ │
│  └───────────┘  └───────────┘  └───────────┘  └──────────────────┘ │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              协议转换层 (Protocol Adapter)                    │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐  │   │
│  │  │ OpenAI 适配器    │  │ Anthropic 适配器  │  │ OpenAI       │  │   │
│  │  │ /chat/completions│  │ /v1/messages     │  │ Responses   │  │   │
│  │  │                 │  │                  │  │ API 适配器  │  │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────┘  │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────┬───────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    路由引擎层 (Router Engine)                        │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                   策略执行管道 (Pipeline)                     │   │
│  │                                                              │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐   │   │
│  │  │ 请求规范化 │→│ 任务分类  │→│ 熔断检查  │→│ 健康评分   │   │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────┘   │   │
│  │                                                              │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐   │   │
│  │  │ 模型筛选  │→│ 评分排序  │→│ 最终选择  │→│ 响应转换   │   │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              智能熔断子系统 (Circuit Breaker)                 │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐   │   │
│  │  │ 实时统计  │  │ 趋势预测  │  │ 主动探测  │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────┬───────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│                 提供商适配层 (Provider Adapters)                     │
│                                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────┐  │
│  │ OpenAI   │  │ Anthropic│  │ Google   │  │ Azure    │  │ 其他  │  │
│  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ ...  │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────┘  │
│                                                                      │
│  每个 Adapter 负责:                                                   │
│  ├── HTTP 客户端管理 (连接池、重试、超时)                             │
│  ├── 请求格式转换 (内部统一格式 → 提供商格式)                         │
│  ├── 响应格式转换 (提供商格式 → 内部统一格式)                         │
│  └── 流式转换 (SSE 事件翻译)                                          │
└──────────────────────────┬───────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│                     数据基础设施层                                    │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐                          │
│  │ DuckDB           │  │ 内存层 (进程内)   │                          │
│  │ (分析数据库)      │  │ (moka+dashmap)   │                          │
│  │                   │  │                  │                          │
│  │ ▪ 调用日志        │  │ ▪ 熔断状态       │                          │
│  │ ▪ 成本统计        │  │ ▪ 速率限制计数   │                          │
│  │ ▪ 模型画像        │  │ ▪ 缓存 (TTL)     │                          │
│  │ ▪ 分析查询        │  │ ▪ 本地计数器     │                          │
│  │                   │  │                  │                          │
│  │ 持久化 · 列式      │  │ 零依赖 · 亚μs   │                          │
│  └──────────────────┘  └──────────────────┘                          │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

### 进程模型

```
┌───────────────────────────────────────────────┐
│             单一进程 (单实例模式)                │
│                                               │
│  ┌─────────────────────────────────────────┐  │
│  │              Rust 二进制                  │  │
│  │                                         │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐ │  │
│  │  │ 路由引擎  │  │ 熔断器   │  │ 适配器   │ │  │
│  │  │ (dashmap)│  │ (moka)  │  │ (DuckDB)│ │  │
│  │  └─────────┘  └─────────┘  └─────────┘ │  │
│  │                                         │  │
│  │  ┌─────────────────────────────────┐   │  │
│  │  │  存储层 (全部进程内)              │   │  │
│  │  │  ├─ JSON 配置 → 文件系统         │   │  │
│  │  │  ├─ moka 缓存 → 进程内存          │   │  │
│  │  │  ├─ dashmap 状态 → 进程内存       │   │  │
│  │  │  └─ DuckDB 日志 → 单文件磁盘      │   │  │
│  │  └─────────────────────────────────┘   │  │
│  └─────────────────────────────────────────┘  │
│                                               │
│  零外部依赖 · 单文件部署 · 即开即用            │
└───────────────────────────────────────────────┘
```

---

## 项目结构

```
llm-smart-routers/
├── Cargo.toml                    # 工作空间根
├── Cargo.lock
│
├── crates/
│   ├── gateway/                  # API 网关层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── server.rs         # HTTP 服务器
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs       # API Key 认证
│   │       │   ├── rate_limit.rs # 速率限制
│   │       │   └── logging.rs    # 请求日志
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── chat_completions.rs
│   │       │   ├── messages.rs
│   │       │   ├── responses.rs
│   │       │   ├── models.rs
│   │       │   └── health.rs
│   │       └── config.rs
│   │
│   ├── router-engine/            # 路由引擎
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pipeline.rs       # 策略执行管道
│   │       ├── classifier.rs     # 任务分类器
│   │       ├── scorer.rs         # 评分引擎
│   │       ├── strategies/
│   │       │   ├── mod.rs
│   │       │   ├── manual.rs
│   │       │   ├── failover.rs
│   │       │   ├── load_balance.rs
│   │       │   ├── cost_optimized.rs
│   │       │   ├── task_aware.rs
│   │       │   ├── scored.rs
│   │       │   └── adaptive.rs
│   │       └── models.rs         # 内部数据模型
│   │
│   ├── circuit-breaker/          # 智能熔断器
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── state.rs          # 熔断状态机
│   │       ├── sliding_window.rs # 滑动窗口统计
│   │       ├── trend_detector.rs # 延迟趋势预测
│   │       ├── health_scorer.rs  # 健康度评分
│   │       ├── prober.rs         # 主动探测
│   │       └── config.rs
│   │
│   ├── provider/                 # 提供商适配层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs         # HTTP 客户端
│   │       ├── registry.rs       # 提供商注册中心
│   │       ├── adapters/
│   │       │   ├── mod.rs
│   │       │   ├── openai.rs     # OpenAI 适配器
│   │       │   ├── anthropic.rs  # Anthropic 适配器
│   │       │   ├── google.rs     # Google AI 适配器
│   │       │   └── azure.rs      # Azure OpenAI 适配器
│   │       └── streaming.rs      # SSE 流式处理
│   │
│   ├── protocol/                 # 协议转换层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── openai.rs         # OpenAI 格式定义
│   │       ├── anthropic.rs      # Anthropic 格式定义
│   │       ├── unified.rs        # 内部统一格式
│   │       ├── converter.rs      # 格式转换器
│   │       └── streaming.rs      # 流式转换
│   │
│   ├── storage/                  # 数据持久化
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── cache.rs           # 缓存层 (moka + dashmap)
│   │       ├── duckdb.rs          # DuckDB 操作 (日志、分析)
│   │       └── json_store.rs      # JSON 文件管理 (配置)
│   │
│   └── sdk/                      # 客户端 SDK (可选)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── client.rs
│
├── webui/                        # 独立 WebUI (可选组件)
│   ├── package.json
│   ├── next.config.js
│   ├── src/
│   │   ├── app/
│   │   │   ├── page.tsx
│   │   │   ├── providers/
│   │   │   ├── strategies/
│   │   │   ├── analytics/
│   │   │   └── settings/
│   │   ├── components/
│   │   └── lib/
│
├── config/
│   ├── default.yaml              # 默认配置
│   ├── circuit_breaker.yaml      # 熔断配置
│   └── strategies.yaml           # 路由策略配置
│
├── docs/
│   ├── architecture.md
│   ├── protocol-conversion.md
│   ├── circuit-breaker.md
│   ├── strategies.md
│   └── api.md
│
├── tests/
│   ├── integration/
│   └── benchmarks/
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
│
└── README.md
```

---

## 开发路线图

### Phase 1: 核心引擎 (MVP) — 8-10 周

| 里程碑 | 时间 | 交付物 |
|--------|------|--------|
| **M1: 项目骨架** | 第 1 周 | Cargo workspace、CI/CD、Docker 环境 |
| **M2: 统一 API 接口** | 第 2-3 周 | 三大协议端点 + 格式转换核心 |
| **M3: 提供商适配器** | 第 4-5 周 | OpenAI、Anthropic 适配器，用户自注册 |
| **M4: 基础熔断器** | 第 6 周 | 实时统计 + 状态机 + 健康度评分 |
| **M5: 基础路由策略** | 第 7 周 | 手动指定、故障转移、负载均衡 |
| **M6: 存储 & 配置** | 第 8 周 | DuckDB + JSON 集成，配置热加载 |
| **M7: 集成测试** | 第 9-10 周 | 端到端测试、性能基准调优 |

### Phase 2: 智能路由 — 6-8 周

| 里程碑 | 时间 | 交付物 |
|--------|------|--------|
| **M8: 任务分类器** | 第 11-12 周 | 请求内容分析，8 种任务类型识别 |
| **M9: 评分路由** | 第 13 周 | 多维度评分引擎，可配置权重 |
| **M10: 智能熔断增强** | 第 14 周 | 延迟趋势预测、主动探测、提供商级联 |
| **M11: 自适应优化** | 第 15 周 | 反馈收集、模型画像自动更新 |
| **M12: 策略插件系统** | 第 16 周 | 自定义策略插件 API |
| **M13: 性能调优** | 第 17-18 周 | 基准测试、P99 延迟优化 |

### Phase 3: 企业级 — 持续

| 特性 | 优先级 | 说明 |
|------|--------|------|
| 提供商级联熔断 | P0 | 多模型、多区域协同熔断 |
| WebUI 管理后台 | P0 | 独立 React 应用 |
| 多租户隔离 | P1 | 工作空间、配额管理 |
| 高级监控告警 | P1 | Prometheus 指标、Grafana 面板 |
| 虚拟 API Key | P1 | 用户自定义 Key 管理 |
| 审计日志 | P2 | 完整调用链追踪 |
| 护栏/内容安全 | P2 | 输入输出过滤 |
| Provider SDK | P2 | 官方 TypeScript/Python SDK |

---

## 快速开始

### 前置要求

- Rust 1.80+
- **无任何外部服务依赖**（DuckDB 嵌入式、moka/dashmap 进程内、JSON 文件配置）

### 本地开发

```bash
# 克隆项目
git clone https://github.com/Maicarons/llm-smart-routers
cd llm-smart-routers

# 直接启动，无需任何外部服务
cargo run -p gateway

# 服务启动在 http://localhost:8080
# DuckDB 自动创建 data/smart_router.duckdb
# JSON 配置存储在 data/config/*.json
```

### 配置示例

```yaml
# config/default.yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4

storage:
  # 分析数据库 (持久化)
  duckdb:
    path: "data/smart_router.duckdb"
    auto_archive: true
    archive_threshold_days: 30

  # 缓存层 (进程内内存)
  cache:
    enabled: true
    ttl_default_sec: 300
    max_capacity: 10000

  # 配置文件 (JSON)
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

### 使用示例

```bash
# 1. 通过 OpenAI 兼容接口调用 (自动路由)
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-your-key-1" \
  -d '{
    "model": "auto",
    "messages": [{"role": "user", "content": "用 Python 写一个快速排序"}],
    "strategy": "cost_optimized"
  }'

# 2. 通过 Anthropic 兼容接口调用
curl http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-your-key-1" \
  -d '{
    "model": "auto",
    "messages": [{"role": "user", "content": "Explain quantum computing"}],
    "max_tokens": 1000
  }'

# 3. 注册自定义提供商
curl -X POST http://localhost:8080/admin/providers \
  -H "Authorization: Bearer admin-key" \
  -d '{
    "name": "my-openai",
    "api_base_url": "https://api.openai.com/v1",
    "api_key": "sk-real-key-here",
    "models": [
      {"id": "gpt-4o", "capabilities": ["chat", "vision"]},
      {"id": "gpt-4o-mini", "capabilities": ["chat"]}
    ]
  }'
```

---

## 数据模型

### 双层存储架构

本系统采用双层存储架构，所有数据都在单进程内管理：

```
┌─────────────────────────────────────────────────────────────────────┐
│  层 1: JSON 配置层 (文件系统)                                        │
│  ─────────────────────                                             │
│  存储内容: 提供商配置、路由策略、API Key、系统设置                    │
│  特点: 人类可读、支持 Git 版本管理、热重载                            │
│  路径: data/config/                                                  │
│                                                                     │
│  providers.json  → 提供商注册信息 (用户编辑)                          │
│  strategies.json → 路由策略配置 (用户编辑)                            │
│  api_keys.json   → API Key 列表 (用户编辑)                           │
│  settings.json   → 系统设置 (自动生成 + 用户编辑)                     │
├─────────────────────────────────────────────────────────────────────┤
│  层 2: 运行时数据层 (进程内)                                         │
│  ─────────────────────                                             │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  内存 (moka + dashmap)                                       │  │
│  │  ─────────────────────                                       │  │
│  │  ▪ 熔断器状态 (provider → health_score)                       │  │
│  │  ▪ 速率限制计数器 (key → count + TTL)                         │  │
│  │  ▪ 响应缓存 (request_hash → response, 自动 TTL 过期)          │  │
│  │  ▪ 统计计数器 (总请求数、成功数、失败数)                        │  │
│  │  优势: 零依赖、亚微秒级访问、进程启动即初始化                    │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  DuckDB (嵌入式列式数据库)                                    │  │
│  │  ──────────────────────────                                  │  │
│  │  ▪ 调用日志 (每次请求的完整记录)                                │  │
│  │  ▪ 成本统计 (按提供商/模型/时间维度聚合)                        │  │
│  │  ▪ 模型画像 (延迟 P50/P95、成功率、质量评分)                   │  │
│  │  ▪ 延迟分布直方图                                             │  │
│  │  优势: 列式分析引擎、单文件持久化、ACID 事务                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### 数据流

```
                          ┌──────────────────┐
                          │  用户请求         │
                          └────────┬─────────┘
                                   │
                                   ▼
                   ┌───────────────────────────────┐
                   │  JSON 配置层 (读取)             │
                   │  ├── 策略配置                  │
                   │  ├── 提供商列表                │
                   │  └── API Key 校验              │
                   └───────────────┬───────────────┘
                                   │
                                   ▼
                   ┌───────────────────────────────┐
                   │  内存层 (读/写)                 │
                   │  ├── 熔断检查 (dashmap)        │
                   │  ├── 限流检查 (dashmap)        │
                   │  └── 缓存查询 (moka)           │
                   └───────────────┬───────────────┘
                                   │
                                   ▼
                   ┌───────────────────────────────┐
                   │  路由决策 & 模型调用            │
                   └───────────────┬───────────────┘
                                   │
                                   ▼
                   ┌───────────────────────────────┐
                   │  DuckDB 分析层 (异步写入)       │
                   │  ├── 记录调用日志              │
                   │  ├── 更新成本统计              │
                   │  └── 更新模型画像              │
                   └───────────────────────────────┘
```

---

## FAQ

### 为什么选择 DuckDB + JSON 而不是传统数据库？

两层架构的核心逻辑：**配置用 JSON，运行时用内存，日志用 DuckDB**。

| 数据类别 | 方案 | 理由 |
|---------|------|------|
| **配置** (提供商、策略、Key) | JSON 文件 | 人类可读，Git 版本管理，热重载，零依赖 |
| **运行时状态** (熔断、限流、缓存) | moka + dashmap (进程内存) | 零依赖，亚微秒级访问，进程启动即就绪 |
| **日志/分析** (调用记录、成本统计) | DuckDB | 列式引擎分析快，嵌入式零运维，单文件备份 |

DuckDB 相比传统数据库（PostgreSQL/SQLite）的核心优势：**列式向量化引擎**（分析查询比 PostgreSQL 快 10-50 倍）、**嵌入式零服务进程**、**单文件备份**（`cp` 即备份）。局限是单写入器，但本项目日志写入通过异步批量队列缓冲后写入，不存在冲突。

### 数据存储在哪里？

| 数据 | 存储方式 | 位置 |
|------|---------|------|
| 配置 (提供商、策略、Key) | JSON 文件 | `data/config/*.json` |
| 运行时状态 (熔断、限流、缓存) | 进程内存 (moka + dashmap) | 进程内 |
| 日志 & 分析 | DuckDB 嵌入式数据库 | `data/smart_router.duckdb` |

所有数据均在进程内或本地文件系统管理，**无需任何外部服务**。

### 数据怎么备份？

```bash
# 配置备份
cp -r data/config/ backup/

# DuckDB 日志备份 (单文件)
cp data/smart_router.duckdb backup/

# 恢复
cp backup/smart_router.duckdb data/
```

### 如何添加新的提供商？

用户通过 REST API 或配置文件注册提供商信息，包括 API 基础 URL、API Key、支持的模型列表和能力标签。系统不需要内置任何提供商 —— 完全由用户自行管理。

### DuckDB 的单写入器限制如何处理？

本项目是单进程架构，DuckDB 的单写入器限制不构成问题。日志写入通过异步批量队列缓冲（1000 条或 1 秒一次批量写入），不会阻塞路由决策路径。

协议转换在 Rust 中通过零拷贝字符串操作和枚举匹配实现，单次转换开销 <10μs。相比 LLM 调用本身的 1-10 秒延迟，转换开销可以忽略不计。

### 智能熔断和传统熔断有什么区别？

传统熔断器（Hystrix 等）只有二值状态（开/关）和固定阈值。我们的五维智能熔断系统引入：健康度渐进式评分（0~1 连续值）、延迟趋势预测（提前发现）、主动探测（快速恢复）和提供商级联（隔离故障传播）。

### WebUI 是必需的吗？

不是。WebUI 是完全独立的可选组件。项目核心是一个后台服务，通过 REST API 和配置文件管理。WebUI 只提供可视化管理界面，方便非技术用户使用。

---

## 参考资料

### 技术参考

- [Cloudflare Pingora](https://github.com/cloudflare/pingora) — Rust 代理框架，设计参考
- [TechEmpower Benchmarks](https://www.techempower.com/benchmarks/) — Web 框架性能基准
- [Tokio](https://tokio.rs) — Rust 异步运行时
- [Axum](https://github.com/tokio-rs/axum) — Web 框架
- [Tower](https://github.com/tower-rs/tower) — 可组合中间件
- [moka](https://github.com/moka-rs/moka) — 高性能并发缓存 (Java Caffeine 移植)
- [dashmap](https://github.com/xacrimon/dashmap) — 无锁并发 HashMap
- [DuckDB](https://duckdb.org) — 嵌入式列式分析数据库
- [Anthropic API Docs](https://docs.anthropic.com/en/api) — Messages API
- [OpenAI API Docs](https://platform.openai.com/docs/api-reference) — Chat Completions / Responses API

### 熔断器参考

- [Netflix Hystrix](https://github.com/Netflix/Hystrix) — 传统熔断器实现
- [Resilience4j](https://resilience4j.readme.io) — Java 熔断器库
- [Alibaba Sentinel](https://sentinelguard.io) — 流量控制组件
- Google SRE 熔断 — 《Site Reliability Engineering》第 21 章
- [AWS 限流与熔断最佳实践](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/)

### 协议标准

- [OpenAI API 规范](https://github.com/openai/openai-openapi)
- [Anthropic API 规范](https://github.com/anthropics/anthropic-api-spec)

---

## 许可证

MIT License