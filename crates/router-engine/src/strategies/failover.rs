use super::super::models::*;
use super::Strategy;
use async_trait::async_trait;
use llm_smart_router_provider::registry::ModelInfo;

/// 故障转移策略：按优先级依次尝试，失败则切换到下一个
pub struct FailoverStrategy {
    pub priority: Vec<String>,
}

impl Default for FailoverStrategy {
    fn default() -> Self {
        Self {
            priority: vec![
                "gpt-4o".to_string(),
                "claude-3-5-sonnet".to_string(),
                "gpt-4o-mini".to_string(),
            ],
        }
    }
}

#[async_trait]
impl Strategy for FailoverStrategy {
    fn name(&self) -> &str {
        "failover"
    }

    async fn select(
        &self,
        models: &[ModelInfo],
        _context: &RouteContext,
    ) -> anyhow::Result<RouteDecision> {
        for preferred in &self.priority {
            if let Some(m) = models.iter().find(|m| m.id == *preferred) {
                return Ok(RouteDecision {
                    provider: m.provider.clone(),
                    model: m.id.clone(),
                    confidence: 1.0,
                    task_type: None,
                });
            }
        }

        // 最后兜底
        if let Some(m) = models.first() {
            return Ok(RouteDecision {
                provider: m.provider.clone(),
                model: m.id.clone(),
                confidence: 0.5,
                task_type: None,
            });
        }

        Err(anyhow::anyhow!("no available models"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_failover_selects_primary() {
        let strategy = FailoverStrategy::default();
        let models = vec![
            ModelInfo {
                id: "gpt-4o".to_string(),
                provider: "openai".to_string(),
                capabilities: vec![],
            },
            ModelInfo {
                id: "claude-3-5-sonnet".to_string(),
                provider: "anthropic".to_string(),
                capabilities: vec![],
            },
        ];
        let context = RouteContext {
            model_hint: "auto".to_string(),
            strategy_name: None,
            max_tokens: None,
            temperature: None,
            task_type: None,
        };
        let decision = strategy.select(&models, &context).await.unwrap();
        assert_eq!(decision.model, "gpt-4o");
    }

    #[tokio::test]
    async fn test_failover_fallback() {
        let strategy = FailoverStrategy::default();
        let models = vec![ModelInfo {
            id: "claude-3-5-sonnet".to_string(),
            provider: "anthropic".to_string(),
            capabilities: vec![],
        }];
        let context = RouteContext {
            model_hint: "auto".to_string(),
            strategy_name: None,
            max_tokens: None,
            temperature: None,
            task_type: None,
        };
        let decision = strategy.select(&models, &context).await.unwrap();
        assert_eq!(decision.model, "claude-3-5-sonnet");
    }
}
