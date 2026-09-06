/// 延迟趋势预测器 (EWMA)
pub struct TrendDetector {
    alpha: f64,
    acceleration_windows: u32,
    p95_threshold_ms: u64,
    last_ema: Option<f64>,
    increasing_count: u32,
}

impl TrendDetector {
    pub fn new(alpha: f64, acceleration_windows: u32, p95_threshold_ms: u64) -> Self {
        Self {
            alpha,
            acceleration_windows,
            p95_threshold_ms,
            last_ema: None,
            increasing_count: 0,
        }
    }

    /// 记录延迟样本并返回趋势状态
    pub fn record(&mut self, latency_ms: f64) -> TrendStatus {
        let ema = match self.last_ema {
            Some(prev) => self.alpha * latency_ms + (1.0 - self.alpha) * prev,
            None => latency_ms,
        };
        self.last_ema = Some(ema);

        // 检测延迟是否持续上升
        if latency_ms > self.p95_threshold_ms as f64 {
            self.increasing_count += 1;
        } else {
            self.increasing_count = 0;
        }

        if self.increasing_count >= self.acceleration_windows {
            TrendStatus::Warning
        } else if latency_ms > self.p95_threshold_ms as f64 {
            TrendStatus::Degraded
        } else {
            TrendStatus::Normal
        }
    }

    pub fn current_ema(&self) -> Option<f64> {
        self.last_ema
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendStatus {
    Normal,
    Degraded,
    Warning,
}

/// 提供商级联熔断器
pub struct ProviderCascading {
    provider_health: dashmap::DashMap<String, f64>,
    model_to_provider: dashmap::DashMap<String, String>,
    cascade_factor: f64,
}

impl ProviderCascading {
    pub fn new(cascade_factor: f64) -> Self {
        Self {
            provider_health: dashmap::DashMap::new(),
            model_to_provider: dashmap::DashMap::new(),
            cascade_factor,
        }
    }

    /// 注册模型到提供商的映射
    pub fn register_model(&self, model: &str, provider: &str) {
        self.model_to_provider
            .insert(model.to_string(), provider.to_string());
    }

    /// 记录模型健康度，级联影响同提供商的其他模型
    pub fn record_model_health(&self, model: &str, health: f64) {
        if let Some(provider) = self.model_to_provider.get(model) {
            let provider_key = provider.value().clone();
            drop(provider);

            // 更新提供商健康度
            let mut entry = self
                .provider_health
                .entry(provider_key.clone())
                .or_insert(1.0);
            *entry = *entry * (1.0 - self.cascade_factor) + health * self.cascade_factor;
        }
    }

    /// 获取模型的有效健康度（考虑提供商级联）
    pub fn effective_health(&self, model: &str) -> f64 {
        let model_health = 1.0; // 默认健康
        if let Some(provider) = self.model_to_provider.get(model) {
            if let Some(ph) = self.provider_health.get(provider.value()) {
                // 模型健康度和提供商健康度加权
                return model_health * 0.7 + *ph * 0.3;
            }
        }
        model_health
    }

    /// 获取提供商健康度
    pub fn provider_health(&self, provider: &str) -> Option<f64> {
        self.provider_health.get(provider).map(|v| *v.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_detector_normal() {
        let mut detector = TrendDetector::new(0.3, 3, 5000);
        for _ in 0..5 {
            let status = detector.record(100.0);
            assert_eq!(status, TrendStatus::Normal);
        }
    }

    #[test]
    fn test_trend_detector_warning() {
        let mut detector = TrendDetector::new(0.3, 3, 5000);
        for _ in 0..4 {
            let status = detector.record(6000.0);
            if status == TrendStatus::Warning {
                return;
            }
        }
        panic!("should have triggered warning");
    }

    #[test]
    fn test_provider_cascading() {
        let pc = ProviderCascading::new(0.3);
        pc.register_model("gpt-4o", "openai");
        pc.register_model("gpt-4o-mini", "openai");

        // gpt-4o 健康度下降
        pc.record_model_health("gpt-4o", 0.2);
        pc.record_model_health("gpt-4o", 0.1);

        // gpt-4o-mini 也应受到影响
        let mini_health = pc.effective_health("gpt-4o-mini");
        assert!(
            mini_health < 1.0,
            "gpt-4o-mini health should be affected: {}",
            mini_health
        );
    }
}
