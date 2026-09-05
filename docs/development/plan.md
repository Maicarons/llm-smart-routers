# LLM Smart Router 完整开发计划

> **For agentic workers:** 本计划使用 `superpowers:subagent-driven-development` 或 `superpowers:executing-plans` 按任务逐项实现。步骤使用复选框语法跟踪进度。

**目标：** 构建一个高性能、多协议统一、智能熔断的 AI 路由引擎，用户通过统一接口接入多个 LLM 提供商，系统实时选择最优模型。

**架构：** 五层架构（API 网关 → 协议转换 → 路由引擎 → 提供商适配 → 数据存储），全部在单进程内运行，零外部依赖。

**技术栈：** Rust + Axum + Tokio + Tower + DuckDB + moka + dashmap + serde_json

**开发周期：** 预计 18-24 周（MVP 8-10 周，智能路由 6-8 周，企业级持续迭代）

---

## 全局约束

- **语言**：所有核心代码使用 Rust（edition 2024），WebUI 使用 TypeScript + React
- **零外部依赖**：不允许依赖 PostgreSQL、Redis、Docker 等外部服务
- **存储**：配置使用 JSON 文件，运行时状态使用 moka/dashmap（进程内存），日志分析使用 DuckDB（嵌入式）
- **协议**：必须支持 OpenAI Chat Completions、OpenAI Responses、Anthropic Messages 三种协议格式
- **提供商**：零内置提供商，用户通过配置文件或 REST API 自行注册
- **WebUI**：完全独立可选组件，核心引擎不依赖它
- **许可证**：AGPL-3.0
- **命名规范**：Rust 使用 snake_case，TypeScript 使用 camelCase，API 路径使用 kebab-case
- **文档**：所有公共 API 必须有 Rustdoc 或 TypeScript JSDoc 注释

---

## 文件结构

```
llm-smart-routers/
├── Cargo.toml                          # 工作空间根
├── Cargo.lock
├── LICENSE                             # AGPL-3.0
├── README.md                           # 项目总览
├── .gitignore
├── .gitattributes
├── .github/
│   └── workflows/
│       ├── ci.yml                      # 持续集成
│       └── release.yml                 # 发布流程
│
├── crates/
│   ├── gateway/                        # API 网关层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                 # 入口
│   │       ├── server.rs               # HTTP 服务器
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs             # API Key 认证
│   │       │   ├── rate_limit.rs       # 速率限制
│   │       │   └── logging.rs          # 请求日志
│   │       └── routes/
│   │           ├── mod.rs
│   │           ├── chat_completions.rs # /v1/chat/completions
│   │           ├── messages.rs         # /v1/messages
│   │           ├── responses.rs        # /v1/responses (OpenAI Responses)
│   │           ├── models.rs           # /v1/models
│   │           └── health.rs           # /health
│   │
│   ├── router-engine/                  # 路由引擎
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pipeline.rs             # 策略执行管道
│   │       ├── classifier.rs           # 任务分类器
│   │       ├── scorer.rs               # 评分引擎
│   │       ├── strategies/
│   │       │   ├── mod.rs
│   │       │   ├── manual.rs           # 手动指定
│   │       │   ├── failover.rs         # 故障转移
│   │       │   ├── load_balance.rs     # 负载均衡
│   │       │   ├── cost_optimized.rs   # 成本优先
│   │       │   ├── task_aware.rs       # 任务感知
│   │       │   ├── scored.rs           # 评分优化
│   │       │   └── adaptive.rs         # 自适应
│   │       └── models.rs              # 内部数据模型
│   │
│   ├── circuit-breaker/                # 智能熔断器
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── state.rs                # 熔断状态机
│   │       ├── sliding_window.rs       # 滑动窗口统计
│   │       ├── trend_detector.rs       # 延迟趋势预测
│   │       ├── health_scorer.rs        # 健康度评分
│   │       ├── prober.rs               # 主动探测
│   │       └── config.rs              # 熔断配置
│   │
│   ├── provider/                       # 提供商适配层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs              # HTTP 客户端
│   │       ├── registry.rs            # 提供商注册中心
│   │       ├── adapters/
│   │       │   ├── mod.rs
│   │       │   ├── openai.rs          # OpenAI 适配器
│   │       │   ├── anthropic.rs       # Anthropic 适配器
│   │       │   ├── google.rs          # Google AI 适配器
│   │       │   └── azure.rs           # Azure OpenAI 适配器
│   │       └── streaming.rs           # SSE 流式处理
│   │
│   ├── protocol/                       # 协议转换层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── openai.rs              # OpenAI 格式定义
│   │       ├── anthropic.rs           # Anthropic 格式定义
│   │       ├── unified.rs             # 内部统一格式
│   │       ├── converter.rs           # 格式转换器
│   │       └── streaming.rs           # 流式转换
│   │
│   └── storage/                        # 数据持久化
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── cache.rs               # 缓存层 (moka + dashmap)
│           ├── duckdb.rs              # DuckDB 操作
│           └── json_store.rs          # JSON 文件管理
│
├── config/                             # 默认配置
│   ├── default.yaml
│   ├── circuit_breaker.yaml
│   └── strategies.yaml
│
├── tests/
│   ├── integration/                    # 集成测试
│   └── benchmarks/                     # 性能基准测试
│
├── docs/                               # VitePress 文档
│   ├── package.json
│   ├── .vitepress/
│   │   └── config.ts
│   ├── index.md
│   ├── guide/
│   │   ├── getting-started.md
│   │   ├── configuration.md
│   │   └── provider-management.md
│   ├── architecture/
│   │   ├── overview.md
│   │   ├── protocol-conversion.md
│   │   ├── circuit-breaker.md
│   │   ├── routing-strategies.md
│   │   └── data-storage.md
│   ├── development/
│   │   ├── plan.md                    # 本文件
│   │   ├── phase-1-core.md
│   │   ├── phase-2-smart-routing.md
│   │   └── phase-3-enterprise.md
│   └── api/
│       ├── reference.md
│       ├── chat-completions.md
│       ├── messages.md
│       ├── responses.md
│       └── admin.md
│
└── webui/                              # 独立 WebUI (可选)
    ├── package.json
    ├── next.config.js
    └── src/
        ├── app/
        ├── components/
        └── lib/
```

---

## 开发路线图

### 总览

```
Phase 1: 核心引擎 (MVP)     Phase 2: 智能路由        Phase 3: 企业级
─────────────────────       ──────────────────       ─────────────────
M1  项目骨架                 M8  任务分类器            P0  提供商级联熔断
M2  统一 API 接口            M9  评分路由              P0  WebUI 管理后台
M3  提供商适配器              M10 智能熔断增强          P1  多租户隔离
M4  基础熔断器               M11 自适应优化            P1  高级监控告警
M5  基础路由策略              M12 策略插件系统          P2  审计日志
M6  存储 & 配置              M13 性能调优              P2  护栏/内容安全
M7  集成测试
├──── 8-10 周 ────┤         ├── 6-8 周 ──┤           └── 持续 ────┤
```

### Phase 1: 核心引擎 (MVP) — 8-10 周

构建最小可行产品：统一 API 接口、基础路由、提供商管理、简易熔断。

| 里程碑 | 时间 | 交付物 |
|--------|------|--------|
| **M1: 项目骨架** | 第 1 周 | Cargo workspace、CI/CD、GitHub 模板、文档框架 |
| **M2: 统一 API 接口** | 第 2-3 周 | 三大协议端点 + 格式转换核心 |
| **M3: 提供商适配器** | 第 4-5 周 | OpenAI、Anthropic 适配器，用户自注册 |
| **M4: 基础熔断器** | 第 6 周 | 实时统计 + 状态机 + 健康度评分 |
| **M5: 基础路由策略** | 第 7 周 | 手动指定、故障转移、负载均衡 |
| **M6: 存储 & 配置** | 第 8 周 | DuckDB + JSON 集成，配置热加载 |
| **M7: 集成测试** | 第 9-10 周 | 端到端测试、性能基准调优 |

### Phase 2: 智能路由 — 6-8 周

添加智能路由能力：任务感知、多维度评分、自适应优化。

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
| 提供商级联熔断 | P0 | 多模型、多提供商协同熔断 |
| WebUI 管理后台 | P0 | 独立 React 应用 |
| 多租户隔离 | P1 | 工作空间、配额管理 |
| 高级监控告警 | P1 | Prometheus 指标、Grafana 面板 |
| 虚拟 API Key | P1 | 用户自定义 Key 管理 |
| 审计日志 | P2 | 完整调用链追踪 |
| 护栏/内容安全 | P2 | 输入输出过滤 |
| 官方 SDK | P2 | TypeScript/Python SDK |

---

## 任务分解

### 任务 1: 项目骨架

**文件：**
- 创建：`Cargo.toml`（工作空间根）
- 创建：`.gitignore`
- 创建：`.github/workflows/ci.yml`
- 创建：`.github/workflows/release.yml`
- 创建：`crates/gateway/Cargo.toml`
- 创建：`crates/router-engine/Cargo.toml`
- 创建：`crates/circuit-breaker/Cargo.toml`
- 创建：`crates/provider/Cargo.toml`
- 创建：`crates/protocol/Cargo.toml`
- 创建：`crates/storage/Cargo.toml`

**接口：**
- 产生：工作空间配置，各 crate 的 Cargo.toml 依赖声明
- 产生：CI 流水线验证 `cargo build --workspace` 通过

- [ ] **Step 1: 创建工作空间 Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/gateway",
    "crates/router-engine",
    "crates/circuit-breaker",
    "crates/provider",
    "crates/protocol",
    "crates/storage",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "AGPL-3.0"
```

- [ ] **Step 2: 创建各 crate 的 Cargo.toml**

每个 crate 的 Cargo.toml 模板：

```toml
[package]
name = "llm-smart-router-{crate_name}"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
# 各 crate 独立声明依赖
```

- [ ] **Step 3: 创建 .gitignore**

```
/target
.env
*.duckdb
data/
node_modules/
dist/
.vitepress/dist/
*.log
```

- [ ] **Step 4: 创建 CI 工作流**

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --check
```

- [ ] **Step 5: 提交并推送**

```bash
git add Cargo.toml .gitignore .github/
git commit -m "chore: initialize workspace structure"
```

---

### 任务 2: 统一 API 接口

**文件：**
- 创建：`crates/protocol/src/lib.rs`
- 创建：`crates/protocol/src/openai.rs`
- 创建：`crates/protocol/src/anthropic.rs`
- 创建：`crates/protocol/src/unified.rs`
- 创建：`crates/protocol/src/converter.rs`
- 创建：`crates/protocol/src/streaming.rs`
- 创建：`crates/gateway/src/main.rs`
- 创建：`crates/gateway/src/server.rs`
- 创建：`crates/gateway/src/routes/mod.rs`
- 创建：`crates/gateway/src/routes/chat_completions.rs`
- 创建：`crates/gateway/src/routes/messages.rs`
- 创建：`crates/gateway/src/routes/responses.rs`
- 创建：`crates/gateway/src/routes/models.rs`
- 创建：`crates/gateway/src/routes/health.rs`

**接口：**
- 消耗：无（独立 crate）
- 产生：`UnifiedRequest`、`UnifiedResponse`、`Converter` trait
- 产生：`POST /v1/chat/completions`、`POST /v1/messages`、`POST /v1/responses` 端点
- 产生：`GET /v1/models`、`GET /health` 端点

- [ ] **Step 1: 定义统一数据模型**

```rust
// crates/protocol/src/unified.rs
pub struct UnifiedRequest {
    pub model: String,
    pub messages: Vec<UnifiedMessage>,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub tools: Vec<UnifiedTool>,
    pub stream: bool,
    pub metadata: HashMap<String, String>,
}

pub struct UnifiedMessage {
    pub role: UnifiedRole,
    pub content: UnifiedContent,
}

pub enum UnifiedRole {
    System,
    User,
    Assistant,
    Tool,
}

pub enum UnifiedContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

pub struct UnifiedResponse {
    pub id: String,
    pub model: String,
    pub content: UnifiedContent,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
    pub stream_events: Vec<StreamEvent>,
}
```

- [ ] **Step 2: 定义 OpenAI 协议格式**

```rust
// crates/protocol/src/openai.rs
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<ChatTool>>,
}

pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}
```

- [ ] **Step 3: 定义 OpenAI Responses 协议格式**

```rust
// crates/protocol/src/responses.rs
pub struct ResponsesRequest {
    pub model: String,
    pub input: ResponseInput,  // 可以是字符串或 InputItem 数组
    pub instructions: Option<String>,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<ResponseTool>>,
    pub store: Option<bool>,
}

pub enum ResponseInput {
    Text(String),
    Items(Vec<InputItem>),
}

pub struct ResponsesResponse {
    pub id: String,
    pub object: String,
    pub status: String,  // "completed" | "incomplete" | "failed"
    pub output: Vec<OutputItem>,
    pub usage: ResponseUsage,
    pub created_at: u64,
}
```

- [ ] **Step 4: 定义 Anthropic 协议格式**

```rust
// crates/protocol/src/anthropic.rs
pub struct MessagesRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    pub system: Option<String>,
    pub max_tokens: u32,
    pub stream: Option<bool>,
}

pub struct MessagesResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: AnthropicUsage,
}
```

- [ ] **Step 5: 实现协议转换器**

```rust
// crates/protocol/src/converter.rs
pub trait Converter {
    fn to_unified_request(&self, bytes: &[u8]) -> Result<UnifiedRequest>;
    fn from_unified_response(&self, resp: &UnifiedResponse) -> Result<Vec<u8>>;
    fn to_stream_event(&self, event: &StreamEvent) -> Result<Vec<u8>>;
}

pub struct OpenAI;
pub struct AnthropicMessages;
pub struct OpenAIResponses;
```

转换规则：
- OpenAI Chat Completions → Unified：`messages[0].role="system"` → `system_prompt`，`choices[0].finish_reason` → `finish_reason`
- OpenAI Responses → Unified：`instructions` → `system_prompt`，`status` → `finish_reason`，`output[0].content[0].text` → 文本内容
- Anthropic Messages → Unified：`system` 顶层字段 → `system_prompt`，`stop_reason` → `finish_reason`
- 流式转换：OpenAI `choices[*].delta` ↔ OpenAI Responses `output` 流式增量 ↔ Anthropic `content_block_delta`

- [ ] **Step 6: 创建 Axum 服务端**

```rust
// crates/gateway/src/main.rs
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/messages", post(messages_handler))
        .route("/v1/responses", post(responses_handler))
        .route("/v1/models", get(models_handler))
        .route("/health", get(health_handler))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 7: 实现请求处理函数**

```rust
// crates/gateway/src/routes/chat_completions.rs
pub async fn chat_completions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // 1. 认证
    // 2. 协议转换 (OpenAI → Unified)
    // 3. 路由决策
    // 4. 调用提供商
    // 5. 协议转换 (Unified → OpenAI)
    // 6. 返回响应
}
```

- [ ] **Step 8: 创建健康检查和模型列表端点**

```rust
// crates/gateway/src/routes/health.rs
pub async fn health_handler() -> Json<Value> {
    Json(json!({ "status": "ok", "version": "0.1.0" }))
}

// crates/gateway/src/routes/models.rs
pub async fn models_handler(
    State(state): State<AppState>,
) -> Json<Vec<ModelInfo>> {
    // 从注册中心获取可用模型列表
}
```

- [ ] **Step 9: 编写集成测试**

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_chat_completions() {
        // 发送 POST /v1/chat/completions
        // 验证响应格式正确
    }
}
```

- [ ] **Step 10: 提交**

```bash
git add crates/protocol/ crates/gateway/
git commit -m "feat: implement unified API interface with protocol conversion"
```

---

### 任务 3: 提供商适配器

**文件：**
- 创建：`crates/provider/src/lib.rs`
- 创建：`crates/provider/src/client.rs`
- 创建：`crates/provider/src/registry.rs`
- 创建：`crates/provider/src/adapters/mod.rs`
- 创建：`crates/provider/src/adapters/openai.rs`
- 创建：`crates/provider/src/adapters/anthropic.rs`
- 创建：`crates/provider/src/streaming.rs`

**接口：**
- 消耗：`UnifiedRequest`、`UnifiedResponse`（来自 protocol crate）
- 产生：`ProviderAdapter` trait、`ProviderRegistry`
- 产生：`ProviderAdapter::chat()` → `Result<UnifiedResponse>`

- [ ] **Step 1: 定义 ProviderAdapter trait**

```rust
// crates/provider/src/lib.rs
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, request: &UnifiedRequest) -> Result<UnifiedResponse>;
    async fn chat_stream(&self, request: &UnifiedRequest) -> Result<BoxStream<Result<StreamEvent>>>;
    fn health_check(&self) -> HealthStatus;
}
```

- [ ] **Step 2: 实现 OpenAI 适配器**

```rust
// crates/provider/src/adapters/openai.rs
pub struct OpenAIAdapter {
    client: HttpClient,
    api_key: String,
    base_url: String,
}

#[async_trait]
impl ProviderAdapter for OpenAIAdapter {
    async fn chat(&self, request: &UnifiedRequest) -> Result<UnifiedResponse> {
        // 1. UnifiedRequest → OpenAI ChatCompletionRequest
        // 2. POST {base_url}/v1/chat/completions
        // 3. ChatCompletionResponse → UnifiedResponse
        // 4. 返回
    }
}
```

- [ ] **Step 3: 实现 Anthropic 适配器**

```rust
// crates/provider/src/adapters/anthropic.rs
pub struct AnthropicAdapter {
    client: HttpClient,
    api_key: String,
    base_url: String,
}

#[async_trait]
impl ProviderAdapter for AnthropicAdapter {
    async fn chat(&self, request: &UnifiedRequest) -> Result<UnifiedResponse> {
        // 1. UnifiedRequest → Anthropic MessagesRequest
        // 2. POST {base_url}/v1/messages
        // 3. MessagesResponse → UnifiedResponse
        // 4. 返回
    }
}
```

- [ ] **Step 4: 实现 ProviderRegistry**

```rust
// crates/provider/src/registry.rs
pub struct ProviderRegistry {
    providers: DashMap<String, Box<dyn ProviderAdapter>>,
    config: PathBuf,  // JSON 配置路径
}

impl ProviderRegistry {
    pub fn new(config_path: PathBuf) -> Self;
    pub fn register(&self, name: &str, adapter: Box<dyn ProviderAdapter>);
    pub fn get(&self, name: &str) -> Option<Box<dyn ProviderAdapter>>;
    pub fn load_from_json(&self) -> Result<()>;
    pub fn list_models(&self) -> Vec<ModelInfo>;
}
```

- [ ] **Step 5: 实现流式处理**

```rust
// crates/provider/src/streaming.rs
pub async fn stream_chat(
    adapter: &dyn ProviderAdapter,
    request: &UnifiedRequest,
) -> BoxStream<Result<StreamEvent>> {
    let stream = adapter.chat_stream(request).await?;
    // 转换流式事件，处理 SSE 格式
    Ok(stream)
}
```

- [ ] **Step 6: 提交**

```bash
git add crates/provider/
git commit -m "feat: implement provider adapters and registry"
```

---

### 任务 4: 基础熔断器

**文件：**
- 创建：`crates/circuit-breaker/src/lib.rs`
- 创建：`crates/circuit-breaker/src/state.rs`
- 创建：`crates/circuit-breaker/src/sliding_window.rs`
- 创建：`crates/circuit-breaker/src/health_scorer.rs`
- 创建：`crates/circuit-breaker/src/config.rs`

**接口：**
- 消耗：`ProviderRegistry`（来自 provider crate）
- 产生：`CircuitBreaker` struct、`HealthScore`、`BreakerState`
- 产生：`CircuitBreaker::check()` → `Result<()>`（允许或拒绝请求）

- [ ] **Step 1: 定义熔断状态机**

```rust
// crates/circuit-breaker/src/state.rs
pub enum BreakerState {
    Closed,        // 全流量
    Warning,       // 预警 (health 0.6-0.8)
    Degraded,      // 降级 (health 0.3-0.6)
    Limited,       // 限制 (health 0.1-0.3)
    Open,          // 熔断 (health < 0.1)
}

pub struct BreakerStatus {
    pub state: BreakerState,
    pub health_score: f64,
    pub last_failure: Option<Instant>,
    pub failure_count: u64,
    pub total_requests: u64,
}
```

- [ ] **Step 2: 实现滑动窗口统计**

```rust
// crates/circuit-breaker/src/sliding_window.rs
pub struct SlidingWindow {
    windows: Vec<TimeBucket>,
    window_size: Duration,
    bucket_count: usize,
}

impl SlidingWindow {
    pub fn record_success(&mut self);
    pub fn record_failure(&mut self, error_type: ErrorType);
    pub fn error_rate(&self) -> f64;
    pub fn latency_p50(&self) -> Duration;
    pub fn latency_p95(&self) -> Duration;
}
```

- [ ] **Step 3: 实现健康度评分**

```rust
// crates/circuit-breaker/src/health_scorer.rs
pub struct HealthScorer {
    config: HealthScoreConfig,
}

impl HealthScorer {
    pub fn score(&self, stats: &CircuitStats) -> f64 {
        let error_score = 1.0 - stats.error_rate;  // 错误率越低分越高
        let latency_score = self.latency_score(stats.latency_p95);
        let trend_score = self.trend_score(stats.trend);
        
        // 加权综合
        error_score * 0.4 + latency_score * 0.3 + trend_score * 0.3
    }
}
```

- [ ] **Step 4: 实现 CircuitBreaker 主结构**

```rust
// crates/circuit-breaker/src/lib.rs
pub struct CircuitBreaker {
    states: DashMap<String, BreakerStatus>,  // model_name → status
    windows: DashMap<String, SlidingWindow>,
    scorer: HealthScorer,
    config: Config,
}

impl CircuitBreaker {
    pub fn check(&self, model: &str) -> Result<()> {
        let status = self.states.get(model).ok_or(Error::NoData)?;
        match status.state {
            BreakerState::Closed | BreakerState::Warning => Ok(()),
            BreakerState::Degraded => Err(Error::Degraded(status.health_score)),
            BreakerState::Limited => Err(Error::Limited(status.health_score)),
            BreakerState::Open => Err(Error::Open),
        }
    }
    
    pub fn record(&self, model: &str, success: bool, latency: Duration, error_type: Option<ErrorType>) {
        // 更新滑动窗口 + 健康度评分 + 状态机迁移
    }
}
```

- [ ] **Step 5: 提交**

```bash
git add crates/circuit-breaker/
git commit -m "feat: implement basic circuit breaker with health scoring"
```

---

### 任务 5: 基础路由策略

**文件：**
- 创建：`crates/router-engine/src/lib.rs`
- 创建：`crates/router-engine/src/pipeline.rs`
- 创建：`crates/router-engine/src/strategies/mod.rs`
- 创建：`crates/router-engine/src/strategies/manual.rs`
- 创建：`crates/router-engine/src/strategies/failover.rs`
- 创建：`crates/router-engine/src/strategies/load_balance.rs`
- 创建：`crates/router-engine/src/models.rs`

**接口：**
- 消耗：`ProviderRegistry`、`CircuitBreaker`
- 产生：`RouterEngine` struct、`Strategy` trait
- 产生：`RouterEngine::route()` → `(ProviderAdapter, ModelInfo)`

- [ ] **Step 1: 定义 Strategy trait**

```rust
// crates/router-engine/src/lib.rs
#[async_trait]
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    async fn select(&self, context: &RouteContext) -> Result<RouteDecision>;
}

pub struct RouteContext {
    pub request: UnifiedRequest,
    pub providers: Vec<ProviderInfo>,
    pub breaker_states: HashMap<String, BreakerStatus>,
}

pub struct RouteDecision {
    pub provider: String,
    pub model: String,
    pub confidence: f64,
}
```

- [ ] **Step 2: 实现手动指定策略**

```rust
// crates/router-engine/src/strategies/manual.rs
pub struct ManualStrategy {
    model: String,
    fallbacks: Vec<String>,
}

#[async_trait]
impl Strategy for ManualStrategy {
    async fn select(&self, _context: &RouteContext) -> Result<RouteDecision> {
        // 直接返回指定模型
        // 如果主模型不可用，依次尝试 fallbacks
    }
}
```

- [ ] **Step 3: 实现故障转移策略**

```rust
// crates/router-engine/src/strategies/failover.rs
pub struct FailoverStrategy {
    primary: String,
    secondaries: Vec<String>,
}

#[async_trait]
impl Strategy for FailoverStrategy {
    async fn select(&self, context: &RouteContext) -> Result<RouteDecision> {
        // 先尝试 primary，如果熔断或失败则尝试 secondary
        for model in std::iter::once(&self.primary).chain(&self.secondaries) {
            if let Ok(()) = context.breaker_states.get(model).map_or(Ok(()), |s| s.check()) {
                return Ok(RouteDecision { provider: model.clone(), model: model.clone(), confidence: 1.0 });
            }
        }
        Err(Error::NoAvailableModel)
    }
}
```

- [ ] **Step 4: 实现负载均衡策略**

```rust
// crates/router-engine/src/strategies/load_balance.rs
pub struct LoadBalanceStrategy {
    models: Vec<String>,
    algorithm: LBAlgorithm,
}

pub enum LBAlgorithm {
    RoundRobin,
    Weighted(HashMap<String, u32>),
    LeastLoad,
}
```

- [ ] **Step 5: 实现 RouterEngine**

```rust
// crates/router-engine/src/pipeline.rs
pub struct RouterEngine {
    strategies: HashMap<String, Box<dyn Strategy>>,
    default_strategy: String,
    breaker: Arc<CircuitBreaker>,
    registry: Arc<ProviderRegistry>,
}

impl RouterEngine {
    pub async fn route(&self, request: &UnifiedRequest, strategy_name: Option<&str>) -> Result<RouteDecision> {
        let name = strategy_name.unwrap_or(&self.default_strategy);
        let strategy = self.strategies.get(name).ok_or(Error::UnknownStrategy)?;
        
        let context = self.build_context(request).await;
        strategy.select(&context).await
    }
}
```

- [ ] **Step 6: 提交**

```bash
git add crates/router-engine/
git commit -m "feat: implement routing strategies and engine"
```

---

### 任务 6: 存储 & 配置

**文件：**
- 创建：`crates/storage/src/lib.rs`
- 创建：`crates/storage/src/cache.rs`
- 创建：`crates/storage/src/duckdb.rs`
- 创建：`crates/storage/src/json_store.rs`
- 创建：`config/default.yaml`
- 创建：`config/circuit_breaker.yaml`
- 创建：`config/strategies.yaml`

**接口：**
- 消耗：`UnifiedResponse`（来自 protocol crate）
- 产生：`Cache`、`DuckDB`、`JsonStore` 实例
- 产生：`Cache::get()` / `Cache::set()` / `DuckDB::log_call()` / `JsonStore::load_providers()`

- [ ] **Step 1: 实现缓存层**

```rust
// crates/storage/src/cache.rs
pub struct Cache {
    response_cache: moka::future::Cache<String, Vec<u8>>,
    counters: dashmap::DashMap<String, (u64, Instant)>,
    breaker_states: dashmap::DashMap<String, BreakerStatus>,
}

impl Cache {
    pub fn new(max_capacity: u64, ttl: Duration) -> Self;
    
    pub async fn get_cached(&self, key: &str) -> Option<Vec<u8>>;
    pub async fn set_cache(&self, key: &str, value: Vec<u8>);
    
    pub fn increment_counter(&self, key: &str, window: Duration) -> u64;
    pub fn get_breaker(&self, key: &str) -> Option<BreakerStatus>;
    pub fn set_breaker(&self, key: &str, status: BreakerStatus);
}
```

- [ ] **Step 2: 实现 DuckDB 持久化**

```rust
// crates/storage/src/duckdb.rs
pub struct AnalyticsDB {
    conn: duckdb::Connection,
}

impl AnalyticsDB {
    pub fn new(path: &str) -> Result<Self>;
    pub fn log_call(&self, call: CallLog) -> Result<()>;
    pub fn query_cost_stats(&self, provider: &str, start: NaiveDate, end: NaiveDate) -> Result<Vec<CostStat>>;
    pub fn query_model_profiles(&self) -> Result<Vec<ModelProfile>>;
    pub fn update_model_profile(&self, model: &str, metrics: Metrics) -> Result<()>;
}
```

DuckDB 表结构：

```sql
CREATE TABLE IF NOT EXISTS call_logs (
    id UUID PRIMARY KEY,
    provider VARCHAR,
    model VARCHAR,
    task_type VARCHAR,
    input_tokens INTEGER,
    output_tokens INTEGER,
    latency_ms INTEGER,
    is_success BOOLEAN,
    error_type VARCHAR,
    cost_usd DOUBLE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS model_profiles (
    provider VARCHAR,
    model VARCHAR,
    p50_latency_ms DOUBLE,
    p95_latency_ms DOUBLE,
    success_rate DOUBLE,
    avg_cost_per_token DOUBLE,
    quality_score DOUBLE,
    total_calls INTEGER,
    updated_at TIMESTAMP,
    PRIMARY KEY (provider, model)
);
```

- [ ] **Step 3: 实现 JSON 配置管理**

```rust
// crates/storage/src/json_store.rs
pub struct JsonStore {
    config_path: PathBuf,
    watcher: Option<FileWatcher>,
}

impl JsonStore {
    pub fn new(path: PathBuf) -> Self;
    pub fn load_providers(&self) -> Result<Vec<ProviderConfig>>;
    pub fn load_strategies(&self) -> Result<Vec<StrategyConfig>>;
    pub fn load_api_keys(&self) -> Result<Vec<ApiKeyConfig>>;
    pub fn save_providers(&self, providers: &[ProviderConfig]) -> Result<()>;
    pub fn watch_updates(&self) -> watch::Receiver<ConfigEvent>;
}
```

JSON 配置示例：

```json
// data/config/providers.json
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

- [ ] **Step 4: 创建默认配置文件**

```yaml
# config/default.yaml
server:
  host: "0.0.0.0"
  port: 8080

storage:
  duckdb:
    path: "data/smart_router.duckdb"
  cache:
    ttl_default_sec: 300
    max_capacity: 10000
  json_config:
    path: "data/config/"
    auto_reload: true
```

- [ ] **Step 5: 提交**

```bash
git add crates/storage/ config/
git commit -m "feat: implement storage layer with DuckDB, cache, and JSON config"
```

---

### 任务 7: 集成测试

**文件：**
- 创建：`tests/integration/routing_test.rs`
- 创建：`tests/integration/protocol_conversion_test.rs`
- 创建：`tests/integration/circuit_breaker_test.rs`
- 创建：`tests/benchmarks/gateway_bench.rs`

- [ ] **Step 1: 协议转换集成测试**

```rust
#[tokio::test]
async fn test_openai_to_anthropic_conversion() {
    let openai_req = r#"{
        "model": "gpt-4o",
        "messages": [{"role": "user", "content": "Hello"}],
        "max_tokens": 100
    }"#;
    
    let converter = OpenAI;
    let unified = converter.to_unified_request(openai_req.as_bytes()).unwrap();
    assert_eq!(unified.messages[0].role, UnifiedRole::User);
    
    // 验证转换回 Anthropic
    let anthropic = AnthropicMessages;
    let anthropic_req = anthropic.from_unified_request(&unified).unwrap();
    // 验证 Anthropic 格式正确
}
```

- [ ] **Step 2: 路由引擎集成测试**

```rust
#[tokio::test]
async fn test_failover_strategy() {
    let engine = RouterEngine::new();
    let request = UnifiedRequest { model: "auto".into(), ... };
    
    // 模拟主模型熔断
    engine.breaker().record("gpt-4o", false, Duration::from_secs(5), Some(ErrorType::Timeout));
    
    let decision = engine.route(&request, Some("failover")).await.unwrap();
    assert_eq!(decision.model, "claude-3-5-sonnet");  // 应切换到备用
}
```

- [ ] **Step 3: 熔断器集成测试**

```rust
#[tokio::test]
async fn test_circuit_breaker_opens_after_threshold() {
    let cb = CircuitBreaker::new(Config::default());
    let model = "test-model";
    
    // 模拟连续失败
    for _ in 0..10 {
        cb.record(model, false, Duration::from_secs(1), Some(ErrorType::Timeout));
    }
    
    // 熔断器应打开
    let result = cb.check(model);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Open));
}
```

- [ ] **Step 4: 性能基准测试**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn gateway_benchmark(c: &mut Criterion) {
    c.bench_function("protocol_conversion_openai", |b| {
        b.iter(|| {
            let converter = OpenAI;
            let _ = converter.to_unified_request(black_box(TEST_REQUEST));
        });
    });
}
```

- [ ] **Step 5: 提交**

```bash
git add tests/
git commit -m "test: add integration tests and benchmarks"
```

---

### Phase 2 任务概览

Phase 2 的详细任务分解见 `docs/development/phase-2-smart-routing.md`，此处仅列出关键任务：

- **任务 8: 任务分类器** — 基于请求内容分析识别 8 种任务类型（代码生成、翻译、创意写作、分析推理、摘要、问答、工具调用、头脑风暴）
- **任务 9: 评分路由** — 多维度评分引擎，可配置权重（能力 0.4、价格 0.2、延迟 0.2、质量 0.15、健康 0.05）
- **任务 10: 智能熔断增强** — 延迟趋势预测（EWMA）、主动健康探测、提供商级联熔断
- **任务 11: 自适应优化** — 反馈收集、模型画像自动更新、路由权重动态调整
- **任务 12: 策略插件系统** — 允许用户通过 WebAssembly 或动态库编写自定义策略
- **任务 13: 性能调优** — P99 延迟优化、内存使用优化、基准测试

---

## 测试策略

### 单元测试
- 每个核心函数必须有单元测试
- 协议转换器：测试每种协议的请求/响应/流式转换
- 熔断器：测试状态机迁移、健康度评分、滑动窗口
- 路由策略：测试每种策略的模型选择逻辑

### 集成测试
- 端到端路由测试：请求 → 协议转换 → 路由 → 提供商调用 → 响应
- 熔断器集成：模拟提供商故障，验证熔断器行为
- 配置热重载：修改 JSON 配置文件，验证系统自动加载

### 性能测试
- 协议转换：单次转换 < 10μs
- 路由决策：单次路由 < 50μs
- 网关吞吐量：> 10,000 req/s（单机）
- 内存使用：100 并发连接时 < 100MB

---

## 发布计划

### v0.1.0 — MVP
- 三大协议端点工作
- 手动指定 + 故障转移路由
- 基础熔断器
- JSON 配置管理
- 基本的调用日志记录

### v0.2.0 — 智能路由
- 任务感知路由
- 评分路由
- 智能熔断增强
- 自适应优化

### v0.3.0 — 企业级
- WebUI 管理后台
- 多租户隔离
- 高级监控告警
- 审计日志

---

## 风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| 协议格式差异过大导致转换复杂 | 高 | 中 | 优先实现核心字段转换，边缘情况逐步补全 |
| 流式 SSE 事件格式不兼容 | 高 | 中 | 设计 Unified Stream Event 作为中间格式 |
| 任务分类准确率不足 | 中 | 低 | 初始使用关键词 + 规则，后续引入 ML 模型 |
| DuckDB 写入性能瓶颈 | 中 | 低 | 异步批量写入，缓冲队列 |
| Rust 编译时间过长 | 低 | 高 | 合理拆分 crate，利用增量编译 |

---

## 执行方式

计划完成并存档到 `docs/development/plan.md`。两种执行方式：

1. **Subagent-Driven（推荐）** — 为每个任务分发独立的子 agent，逐任务审查，快速迭代
2. **Inline Execution** — 在当前会话中执行任务，使用 `superpowers:executing-plans`，批量执行 + 检查点审查

**请审阅本计划，确认后我将开始执行。**