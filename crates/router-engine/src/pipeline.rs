use std::collections::HashMap;
use std::sync::Arc;
use llm_smart_router_protocol::UnifiedRequest;
use llm_smart_router_circuit_breaker::CircuitBreaker;
use llm_smart_router_provider::registry::{ProviderRegistry, ModelInfo};
use super::models::*;
use super::strategies::{
    Strategy,
    manual::ManualStrategy,
    failover::FailoverStrategy,
    load_balance::LoadBalanceStrategy,
};

/// 路由引擎
pub struct RouterEngine {
    strategies: HashMap<String, Box<dyn Strategy>>,
    default_strategy: String,
    pub breaker: Arc<CircuitBreaker>,
    pub registry: Arc<ProviderRegistry>,
}

impl RouterEngine {
    pub fn new(
        breaker: Arc<CircuitBreaker>,
        registry: Arc<ProviderRegistry>,
    ) -> Self {
        let mut strategies: HashMap<String, Box<dyn Strategy>> = HashMap::new();
        strategies.insert("manual".to_string(), Box::new(ManualStrategy::default()));
        strategies.insert("failover".to_string(), Box::new(FailoverStrategy::default()));
        strategies.insert("load_balance".to_string(), Box::new(LoadBalanceStrategy::default()));

        Self {
            strategies,
            default_strategy: "failover".to_string(),
            breaker,
            registry,
        }
    }

    /// 执行路由决策
    pub async fn route(
        &self,
        _request: &UnifiedRequest,
        context: &RouteContext,
    ) -> anyhow::Result<RouteDecision> {
        let strategy_name = context.strategy_name.as_deref().unwrap_or(&self.default_strategy);
        let strategy = self.strategies.get(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("unknown strategy: {}", strategy_name))?;

        let models = self.registry.list_models();
        if models.is_empty() {
            return Err(anyhow::anyhow!("no providers registered"));
        }

        let available_models: Vec<ModelInfo> = models.into_iter()
            .filter(|m| self.breaker.check(&m.id).is_ok())
            .collect();

        if available_models.is_empty() {
            return Err(anyhow::anyhow!("no available models"));
        }

        strategy.select(&available_models, context).await
    }

    /// 注册自定义策略
    pub fn register_strategy(&mut self, name: &str, strategy: Box<dyn Strategy>) {
        self.strategies.insert(name.to_string(), strategy);
    }
}