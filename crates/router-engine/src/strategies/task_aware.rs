use async_trait::async_trait;
use llm_smart_router_provider::registry::ModelInfo;
use super::super::models::*;
use super::super::scorer::{Scorer, ModelCapabilityProfile, ScoreWeights};
use super::super::classifier::TaskType;
use super::Strategy;

/// 任务感知评分策略：根据任务类型选择最优模型
pub struct TaskAwareStrategy {
    scorer: Scorer,
}

impl Default for TaskAwareStrategy {
    fn default() -> Self {
        Self {
            scorer: Scorer::new(ScoreWeights::default()),
        }
    }
}

impl TaskAwareStrategy {
    pub fn new(weights: ScoreWeights) -> Self {
        Self {
            scorer: Scorer::new(weights),
        }
    }
}

#[async_trait]
impl Strategy for TaskAwareStrategy {
    fn name(&self) -> &str {
        "task_aware"
    }

    async fn select(&self, models: &[ModelInfo], context: &RouteContext) -> anyhow::Result<RouteDecision> {
        if models.is_empty() {
            return Err(anyhow::anyhow!("no available models"));
        }

        // 推断任务类型
        let task_type = context.task_type;

        // 为每个模型评分
        let mut scored: Vec<(f64, &ModelInfo)> = models.iter()
            .map(|m| {
                let profile = ModelCapabilityProfile::estimate(&m.id, &m.provider);
                let score = self.scorer.score(&profile, task_type);
                (score, m)
            })
            .collect();

        // 按评分降序排列
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((score, best)) = scored.first() {
            Ok(RouteDecision {
                provider: best.provider.clone(),
                model: best.id.clone(),
                confidence: *score,
                task_type,
            })
        } else {
            Err(anyhow::anyhow!("no available models"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_aware_selects_best_model() {
        let strategy = TaskAwareStrategy::default();
        let models = vec![
            ModelInfo { id: "gpt-4o".to_string(), provider: "openai".to_string(), capabilities: vec![] },
            ModelInfo { id: "gpt-3.5-turbo".to_string(), provider: "openai".to_string(), capabilities: vec![] },
        ];
        let context = RouteContext {
            model_hint: "auto".to_string(),
            strategy_name: None,
            max_tokens: None,
            temperature: None,
            task_type: Some(TaskType::CodeGeneration),
        };
        let decision = strategy.select(&models, &context).await.unwrap();
        // 代码生成任务应该选择 gpt-4o
        assert_eq!(decision.model, "gpt-4o");
    }

    #[tokio::test]
    async fn test_task_aware_cost_sensitive() {
        let strategy = TaskAwareStrategy::new(ScoreWeights::cost_sensitive());
        let gpt4 = ModelCapabilityProfile::estimate("gpt-4o", "openai");
        let gpt35 = ModelCapabilityProfile::estimate("gpt-3.5-turbo", "openai");

        let score_4o = strategy.scorer.score(&gpt4, Some(TaskType::GeneralQa));
        let score_35 = strategy.scorer.score(&gpt35, Some(TaskType::GeneralQa));
        // 成本敏感模式下，便宜模型得分应接近或超过贵模型
        assert!(score_35 >= score_4o * 0.7, "cheap model ({}) should not be too far behind expensive ({})", score_35, score_4o);
    }
}