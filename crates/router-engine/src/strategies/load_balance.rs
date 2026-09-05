use std::sync::atomic::{AtomicUsize, Ordering};
use async_trait::async_trait;
use llm_smart_router_provider::registry::ModelInfo;
use super::super::models::*;
use super::Strategy;

/// 负载均衡策略：在可用模型之间轮询分配
pub struct LoadBalanceStrategy {
    counter: AtomicUsize,
}

impl Default for LoadBalanceStrategy {
    fn default() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Strategy for LoadBalanceStrategy {
    fn name(&self) -> &str {
        "load_balance"
    }

    async fn select(&self, models: &[ModelInfo], _context: &RouteContext) -> anyhow::Result<RouteDecision> {
        if models.is_empty() {
            return Err(anyhow::anyhow!("no available models"));
        }

        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % models.len();
        let m = &models[idx];

        Ok(RouteDecision {
            provider: m.provider.clone(),
            model: m.id.clone(),
            confidence: 0.7,
            task_type: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_balance_round_robin() {
        let strategy = LoadBalanceStrategy::default();
        let models = vec![
            ModelInfo { id: "gpt-4o".to_string(), provider: "openai".to_string(), capabilities: vec![] },
            ModelInfo { id: "claude-3-5-sonnet".to_string(), provider: "anthropic".to_string(), capabilities: vec![] },
        ];
        let context = RouteContext {
            model_hint: "auto".to_string(),
            strategy_name: None,
            max_tokens: None,
            temperature: None,
            task_type: None,
        };

        let d1 = strategy.select(&models, &context).await.unwrap();
        let d2 = strategy.select(&models, &context).await.unwrap();
        assert_ne!(d1.model, d2.model);
    }
}