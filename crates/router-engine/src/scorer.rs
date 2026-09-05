use serde::{Deserialize, Serialize};
use super::classifier::TaskType;

/// 评分因子权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub capability_weight: f64,
    pub price_weight: f64,
    pub latency_weight: f64,
    pub quality_weight: f64,
    pub health_weight: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            capability_weight: 0.4,
            price_weight: 0.2,
            latency_weight: 0.2,
            quality_weight: 0.15,
            health_weight: 0.05,
        }
    }
}

impl ScoreWeights {
    /// 成本敏感预设
    pub fn cost_sensitive() -> Self {
        Self {
            capability_weight: 0.2,
            price_weight: 0.5,
            latency_weight: 0.1,
            quality_weight: 0.1,
            health_weight: 0.1,
        }
    }

    /// 质量优先预设
    pub fn quality_first() -> Self {
        Self {
            capability_weight: 0.4,
            price_weight: 0.1,
            latency_weight: 0.1,
            quality_weight: 0.3,
            health_weight: 0.1,
        }
    }

    /// 速度优先预设
    pub fn speed_first() -> Self {
        Self {
            capability_weight: 0.2,
            price_weight: 0.1,
            latency_weight: 0.5,
            quality_weight: 0.1,
            health_weight: 0.1,
        }
    }
}

/// 模型能力画像
#[derive(Debug, Clone)]
pub struct ModelCapabilityProfile {
    pub model_id: String,
    pub provider: String,
    pub price_per_million_input: f64,
    pub price_per_million_output: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub success_rate: f64,
    pub quality_score: f64,
    pub health_score: f64,
    pub task_scores: Vec<(TaskType, f64)>,  // 各任务类型的适用性评分
}

impl ModelCapabilityProfile {
    /// 默认能力画像（基于模型名称的估计值）
    pub fn estimate(model_id: &str, provider: &str) -> Self {
        let (base_score, price, latency) = match model_id {
            // OpenAI 模型
            id if id.contains("gpt-4o") => (0.95, 2.50, 800.0),
            id if id.contains("gpt-4-turbo") => (0.90, 10.0, 1500.0),
            id if id.contains("gpt-4") => (0.85, 30.0, 2000.0),
            id if id.contains("gpt-3.5-turbo") => (0.70, 0.50, 400.0),
            // Anthropic 模型
            id if id.contains("claude-3-5-sonnet") => (0.92, 3.0, 1000.0),
            id if id.contains("claude-3-opus") => (0.95, 15.0, 2000.0),
            id if id.contains("claude-3-sonnet") => (0.85, 3.0, 800.0),
            id if id.contains("claude-3-haiku") => (0.70, 0.25, 300.0),
            // 默认
            _ => (0.80, 5.0, 1000.0),
        };

        let task_scores = Self::estimate_task_scores(model_id, base_score);

        Self {
            model_id: model_id.to_string(),
            provider: provider.to_string(),
            price_per_million_input: price,
            price_per_million_output: price * 3.0,
            p50_latency_ms: latency,
            p95_latency_ms: latency * 2.0,
            success_rate: 0.98,
            quality_score: base_score,
            health_score: 1.0,
            task_scores,
        }
    }

    fn estimate_task_scores(model_id: &str, base: f64) -> Vec<(TaskType, f64)> {
        let mut scores = Vec::new();
        for task in TaskType::all() {
            let score = match task {
                TaskType::CodeGeneration => {
                    if model_id.contains("gpt-4") || model_id.contains("claude-3") {
                        (base * 0.95).min(1.0)
                    } else {
                        base * 0.7
                    }
                }
                TaskType::CodeExplanation => {
                    if model_id.contains("gpt-4") || model_id.contains("claude-3") {
                        (base * 0.95).min(1.0)
                    } else {
                        base * 0.7
                    }
                }
                TaskType::Translation => base * 0.9,
                TaskType::CreativeWriting => {
                    if model_id.contains("claude") || model_id.contains("gpt-4") {
                        (base * 0.95).min(1.0)
                    } else {
                        base * 0.7
                    }
                }
                TaskType::Analysis => {
                    if model_id.contains("gpt-4") || model_id.contains("claude-3") {
                        (base * 0.95).min(1.0)
                    } else {
                        base * 0.6
                    }
                }
                TaskType::Summarization => base * 0.85,
                TaskType::GeneralQa => base * 0.9,
                TaskType::ToolUse => {
                    if model_id.contains("gpt-4") || model_id.contains("claude-3-5") {
                        (base * 0.95).min(1.0)
                    } else {
                        base * 0.6
                    }
                }
                TaskType::Brainstorming => base * 0.85,
            };
            scores.push((task, score));
        }
        scores
    }
}

/// 评分引擎
pub struct Scorer {
    pub weights: ScoreWeights,
}

impl Scorer {
    pub fn new(weights: ScoreWeights) -> Self {
        Self { weights }
    }

    /// 计算模型总分
    pub fn score(&self, profile: &ModelCapabilityProfile, task_type: Option<TaskType>) -> f64 {
        let capability = task_type
            .and_then(|t| profile.task_scores.iter().find(|(task, _)| *task == t).map(|(_, s)| *s))
            .unwrap_or(profile.quality_score);

        let price = self.normalize_price(profile.price_per_million_input);
        let latency = self.normalize_latency(profile.p50_latency_ms);

        let total = capability * self.weights.capability_weight
            + price * self.weights.price_weight
            + latency * self.weights.latency_weight
            + profile.quality_score * self.weights.quality_weight
            + profile.health_score * self.weights.health_weight;

        total.clamp(0.0, 1.0)
    }

    /// 价格归一化 (越便宜分数越高)
    fn normalize_price(&self, price_per_million: f64) -> f64 {
        if price_per_million <= 0.0 {
            return 1.0;
        }
        // 假设 30 美元/百万 token 是最贵的
        (1.0 - (price_per_million / 30.0).min(1.0)).max(0.0)
    }

    /// 延迟归一化 (越快分数越高)
    fn normalize_latency(&self, latency_ms: f64) -> f64 {
        if latency_ms <= 0.0 {
            return 1.0;
        }
        // 假设 5 秒是最慢的
        (1.0 - (latency_ms / 5000.0).min(1.0)).max(0.0)
    }
}

impl Default for Scorer {
    fn default() -> Self {
        Self::new(ScoreWeights::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_default_weights() {
        let scorer = Scorer::default();
        let profile = ModelCapabilityProfile::estimate("gpt-4o", "openai");
        let score = scorer.score(&profile, Some(TaskType::CodeGeneration));
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_gpt4o_scores_higher_than_gpt35() {
        let scorer = Scorer::new(ScoreWeights::quality_first());
        let gpt4 = ModelCapabilityProfile::estimate("gpt-4o", "openai");
        let gpt35 = ModelCapabilityProfile::estimate("gpt-3.5-turbo", "openai");

        let score_4o = scorer.score(&gpt4, Some(TaskType::CodeGeneration));
        let score_35 = scorer.score(&gpt35, Some(TaskType::CodeGeneration));
        assert!(score_4o > score_35, "gpt-4o ({}) should score higher than gpt-3.5 ({})", score_4o, score_35);
    }

    #[test]
    fn test_cost_sensitive_prefers_cheaper() {
        let scorer = Scorer::new(ScoreWeights::cost_sensitive());
        let expensive = ModelCapabilityProfile::estimate("gpt-4-turbo", "openai");
        let cheap = ModelCapabilityProfile::estimate("gpt-3.5-turbo", "openai");

        let score_expensive = scorer.score(&expensive, Some(TaskType::GeneralQa));
        let score_cheap = scorer.score(&cheap, Some(TaskType::GeneralQa));
        assert!(score_cheap > score_expensive, "cheaper model ({}) should score higher in cost mode ({})", score_cheap, score_expensive);
    }

    #[test]
    fn test_task_scores_differ() {
        let profile = ModelCapabilityProfile::estimate("gpt-4o", "openai");
        let code_score = profile.task_scores.iter()
            .find(|(t, _)| *t == TaskType::CodeGeneration)
            .map(|(_, s)| *s)
            .unwrap();
        let qa_score = profile.task_scores.iter()
            .find(|(t, _)| *t == TaskType::GeneralQa)
            .map(|(_, s)| *s)
            .unwrap();
        // 代码能力应该高于通用 QA
        assert!(code_score >= qa_score);
    }
}