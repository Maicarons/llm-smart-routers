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
    task_aware::TaskAwareStrategy,
};
use super::classifier::Classifier;

/// 路由引擎
pub struct RouterEngine {
    strategies: HashMap<String, Box<dyn Strategy>>,
    default_strategy: String,
    pub breaker: Arc<CircuitBreaker>,
    pub registry: Arc<ProviderRegistry>,
    pub classifier: Classifier,
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
        strategies.insert("task_aware".to_string(), Box::new(TaskAwareStrategy::default()));

        Self {
            strategies,
            default_strategy: "manual".to_string(),
            breaker,
            registry,
            classifier: Classifier::new(),
        }
    }

    /// 执行路由决策
    pub async fn route(
        &self,
        request: &UnifiedRequest,
        context: &RouteContext,
    ) -> anyhow::Result<RouteDecision> {
        let strategy_name = context.strategy_name.as_deref().unwrap_or(&self.default_strategy);
        let strategy = self.strategies.get(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("unknown strategy: {}", strategy_name))?;

        let models = self.registry.list_models();
        if models.is_empty() {
            return Err(anyhow::anyhow!("no providers registered"));
        }

        // 分析任务类型
        let user_text = request.messages.iter()
            .filter(|m| matches!(m.role, llm_smart_router_protocol::UnifiedRole::User))
            .map(|m| match &m.content {
                llm_smart_router_protocol::UnifiedContent::Text(t) => t.clone(),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join(" ");

        let classification = self.classifier.classify(&user_text);
        tracing::debug!("task classification: {:?} (confidence: {:.2})", classification.primary_type.name(), classification.confidence);

        let available_models: Vec<ModelInfo> = models.into_iter()
            .filter(|m| self.breaker.check(&m.id).is_ok())
            .collect();

        if available_models.is_empty() {
            return Err(anyhow::anyhow!("no available models"));
        }

        let mut decision = strategy.select(&available_models, context).await?;
        // 注入任务类型信息
        decision.task_type = Some(classification.primary_type);

        Ok(decision)
    }

    /// 注册自定义策略
    pub fn register_strategy(&mut self, name: &str, strategy: Box<dyn Strategy>) {
        self.strategies.insert(name.to_string(), strategy);
    }
}