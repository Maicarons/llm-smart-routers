use serde::{Deserialize, Serialize};

/// 路由决策
#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub provider: String,
    pub model: String,
    pub confidence: f64,
}

/// 路由上下文
#[derive(Debug, Clone)]
pub struct RouteContext {
    pub model_hint: String,
    pub strategy_name: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// 策略配置（从 JSON 加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub strategy_type: String,
    pub config: Option<serde_json::Value>,
}