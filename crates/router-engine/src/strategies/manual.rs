use async_trait::async_trait;
use llm_smart_router_provider::registry::ModelInfo;
use super::super::models::*;
use super::Strategy;

/// 手动指定策略：使用用户指定的模型，失败时尝试 fallback
pub struct ManualStrategy {
    pub model: String,
    pub fallbacks: Vec<String>,
}

impl Default for ManualStrategy {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            fallbacks: vec!["claude-3-5-sonnet".to_string()],
        }
    }
}

#[async_trait]
impl Strategy for ManualStrategy {
    fn name(&self) -> &str {
        "manual"
    }

    async fn select(&self, models: &[ModelInfo], context: &RouteContext) -> anyhow::Result<RouteDecision> {
        let target = if context.model_hint.is_empty() || context.model_hint == "auto" {
            &self.model
        } else {
            &context.model_hint
        };

        // 先尝试指定模型
        if let Some(m) = models.iter().find(|m| m.id == *target) {
            return Ok(RouteDecision {
                provider: m.provider.clone(),
                model: m.id.clone(),
                confidence: 1.0,
            });
        }

        // 尝试 fallbacks
        for fallback in &self.fallbacks {
            if let Some(m) = models.iter().find(|m| m.id == *fallback) {
                return Ok(RouteDecision {
                    provider: m.provider.clone(),
                    model: m.id.clone(),
                    confidence: 0.8,
                });
            }
        }

        // 使用第一个可用模型
        if let Some(m) = models.first() {
            return Ok(RouteDecision {
                provider: m.provider.clone(),
                model: m.id.clone(),
                confidence: 0.5,
            });
        }

        Err(anyhow::anyhow!("no available models"))
    }
}