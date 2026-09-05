use std::collections::HashMap;
use super::classifier::TaskType;
use super::scorer::ModelCapabilityProfile;

/// 调用反馈
#[derive(Debug, Clone)]
pub struct CallFeedback {
    pub model: String,
    pub provider: String,
    pub task_type: Option<TaskType>,
    pub latency_ms: u64,
    pub success: bool,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub user_rating: Option<u8>,  // 1-5 用户评分，None 表示无评分
}

/// 模型画像缓存
#[derive(Debug, Clone)]
pub struct ModelStats {
    pub total_calls: u64,
    pub success_count: u64,
    pub total_latency: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub quality_score: f64,
    pub task_type_counts: HashMap<TaskType, u64>,
}

/// 自适应优化器
pub struct AdaptiveOptimizer {
    stats: dashmap::DashMap<String, ModelStats>,
    task_performance: dashmap::DashMap<(String, TaskType), f64>,  // (model, task) → score
}

impl AdaptiveOptimizer {
    pub fn new() -> Self {
        Self {
            stats: dashmap::DashMap::new(),
            task_performance: dashmap::DashMap::new(),
        }
    }

    /// 记录一次调用反馈
    pub fn record_feedback(&self, feedback: &CallFeedback) {
        let mut entry = self.stats.entry(feedback.model.clone()).or_insert_with(|| ModelStats {
            total_calls: 0,
            success_count: 0,
            total_latency: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            quality_score: 0.5,
            task_type_counts: HashMap::new(),
        });

        let stats = entry.value_mut();
        stats.total_calls += 1;
        if feedback.success {
            stats.success_count += 1;
        }
        stats.total_latency += feedback.latency_ms;
        stats.total_input_tokens += feedback.input_tokens as u64;
        stats.total_output_tokens += feedback.output_tokens as u64;

        if let Some(task_type) = &feedback.task_type {
            let key = (feedback.model.clone(), *task_type);
            let mut task_entry = self.task_performance.entry(key).or_insert(0.5);
            let rating = feedback.user_rating.unwrap_or(if feedback.success { 4 } else { 1 }) as f64;
            *task_entry = *task_entry * 0.9 + (rating / 5.0) * 0.1;
        }
    }

    /// 获取模型在特定任务上的调整后评分
    pub fn adjusted_score(&self, profile: &ModelCapabilityProfile, task_type: Option<TaskType>) -> f64 {
        let mut score = profile.quality_score;

        if let Some(task) = task_type {
            let key = (profile.model_id.clone(), task);
            if let Some(perf) = self.task_performance.get(&key) {
                score = score * 0.6 + *perf * 0.4;
            }
        }

        if let Some(stats) = self.stats.get(&profile.model_id) {
            if stats.total_calls > 10 {
                let actual_success_rate = stats.success_count as f64 / stats.total_calls as f64;
                score = score * 0.7 + actual_success_rate * 0.3;
            }
        }

        score.clamp(0.0, 1.0)
    }

    pub fn model_stats(&self, model: &str) -> Option<ModelStats> {
        self.stats.get(model).map(|e| e.value().clone())
    }

    pub fn task_score(&self, model: &str, task: TaskType) -> Option<f64> {
        self.task_performance.get(&(model.to_string(), task)).map(|v| *v.value())
    }
}

impl Default for AdaptiveOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::TaskType;

    #[test]
    fn test_adaptive_optimizer_tracks_stats() {
        let optimizer = AdaptiveOptimizer::default();
        optimizer.record_feedback(&CallFeedback {
            model: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            task_type: Some(TaskType::CodeGeneration),
            latency_ms: 500,
            success: true,
            input_tokens: 100,
            output_tokens: 50,
            user_rating: Some(5),
        });

        let stats = optimizer.model_stats("gpt-4o").unwrap();
        assert_eq!(stats.total_calls, 1);
        assert_eq!(stats.success_count, 1);
    }

    #[test]
    fn test_adaptive_optimizer_adjusts_score() {
        let optimizer = AdaptiveOptimizer::default();
        let profile = ModelCapabilityProfile::estimate("gpt-4o", "openai");

        for _ in 0..5 {
            optimizer.record_feedback(&CallFeedback {
                model: "gpt-4o".to_string(),
                provider: "openai".to_string(),
                task_type: Some(TaskType::CodeGeneration),
                latency_ms: 200,
                success: true,
                input_tokens: 50,
                output_tokens: 30,
                user_rating: Some(5),
            });
        }

        let adjusted = optimizer.adjusted_score(&profile, Some(TaskType::CodeGeneration));
        assert!(adjusted > 0.0);
        assert!(adjusted <= 1.0);
    }

    #[test]
    fn test_adaptive_optimizer_records_task_performance() {
        let optimizer = AdaptiveOptimizer::default();
        optimizer.record_feedback(&CallFeedback {
            model: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            task_type: Some(TaskType::Translation),
            latency_ms: 300,
            success: true,
            input_tokens: 80,
            output_tokens: 60,
            user_rating: Some(4),
        });

        let score = optimizer.task_score("gpt-4o", TaskType::Translation);
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }
}