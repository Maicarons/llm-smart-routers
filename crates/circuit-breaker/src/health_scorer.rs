use super::sliding_window::WindowStats;
use serde::{Deserialize, Serialize};

/// 健康度评分配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScoreConfig {
    pub error_rate_weight: f64,
    pub latency_weight: f64,
    pub trend_weight: f64,
    pub slo_weight: f64,
}

impl Default for HealthScoreConfig {
    fn default() -> Self {
        Self {
            error_rate_weight: 0.4,
            latency_weight: 0.3,
            trend_weight: 0.2,
            slo_weight: 0.1,
        }
    }
}

/// 健康度评分器
pub struct HealthScorer {
    config: HealthScoreConfig,
    prev_error_rates: dashmap::DashMap<String, f64>,
}

impl HealthScorer {
    pub fn new(config: HealthScoreConfig) -> Self {
        Self {
            config,
            prev_error_rates: dashmap::DashMap::new(),
        }
    }

    /// 计算综合健康度评分 (0.0 ~ 1.0)
    pub fn score(&self, stats: &WindowStats) -> f64 {
        let error_score = self.error_score(stats.error_rate);
        let latency_score = self.latency_score(stats.p95_latency);
        let trend_score = self.trend_score(stats.error_rate);

        let health = error_score * self.config.error_rate_weight
            + latency_score * self.config.latency_weight
            + trend_score * self.config.trend_weight;

        health.clamp(0.0, 1.0)
    }

    fn error_score(&self, error_rate: f64) -> f64 {
        if error_rate == 0.0 {
            1.0
        } else {
            (1.0 - error_rate).max(0.0)
        }
    }

    fn latency_score(&self, p95_ms: u64) -> f64 {
        if p95_ms == 0 {
            return 1.0;
        }
        // 假设 SLO 是 5 秒，超过则扣分
        let slo_ms = 5000;
        if p95_ms <= slo_ms {
            1.0 - (p95_ms as f64 / slo_ms as f64) * 0.3
        } else {
            // 超过 SLO 大幅扣分
            (1.0 - (p95_ms as f64 / slo_ms as f64).min(5.0) * 0.2).max(0.0)
        }
    }

    fn trend_score(&self, current_error_rate: f64) -> f64 {
        // 如果没有历史数据，给中等分数
        0.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_health_score() {
        let scorer = HealthScorer::new(HealthScoreConfig::default());
        let stats = WindowStats {
            total_requests: 100,
            success_count: 95,
            failure_count: 5,
            error_rate: 0.05,
            p50_latency: 200,
            p95_latency: 1000,
            avg_latency: 300,
            error_type_counts: HashMap::new(),
        };
        let score = scorer.score(&stats);
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_high_error_rate() {
        let scorer = HealthScorer::new(HealthScoreConfig::default());
        let stats = WindowStats {
            total_requests: 100,
            success_count: 20,
            failure_count: 80,
            error_rate: 0.8,
            p50_latency: 5000,
            p95_latency: 10000,
            avg_latency: 6000,
            error_type_counts: HashMap::new(),
        };
        let score = scorer.score(&stats);
        // 高错误率 + 高延迟，评分应很低
        assert!(score < 0.5, "expected score < 0.5, got {}", score);
    }
}
