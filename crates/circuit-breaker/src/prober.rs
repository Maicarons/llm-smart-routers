use std::time::Duration;

/// 主动健康探测结果
#[derive(Debug, Clone)]
pub enum ProbeResult {
    Success { latency_ms: u64 },
    Failure { error: String },
}

/// 主动健康探测器
pub struct Prober {
    config: super::config::ProbingConfig,
    consecutive_successes: dashmap::DashMap<String, u32>,
}

impl Prober {
    pub fn new(config: super::config::ProbingConfig) -> Self {
        Self {
            config,
            consecutive_successes: dashmap::DashMap::new(),
        }
    }

    /// 获取探测间隔
    pub fn interval(&self, model: &str) -> Duration {
        let is_healthy = self.consecutive_successes
            .get(model)
            .map(|c| *c >= self.config.recovery_success_count)
            .unwrap_or(false);

        if is_healthy {
            Duration::from_secs(self.config.interval_healthy_sec)
        } else {
            Duration::from_secs(self.config.interval_unstable_sec)
        }
    }

    /// 探测超时
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.config.probe_timeout_ms)
    }

    /// 记录探测结果
    pub fn record_result(&self, model: &str, result: &ProbeResult) {
        match result {
            ProbeResult::Success { .. } => {
                let mut count = self.consecutive_successes.entry(model.to_string()).or_insert(0);
                *count += 1;
            }
            ProbeResult::Failure { .. } => {
                self.consecutive_successes.insert(model.to_string(), 0);
            }
        }
    }

    /// 检查是否已恢复（达到连续成功次数）
    pub fn is_recovered(&self, model: &str) -> bool {
        self.consecutive_successes
            .get(model)
            .map(|c| *c >= self.config.recovery_success_count)
            .unwrap_or(false)
    }

    /// 获取恢复所需成功次数
    pub fn recovery_count(&self) -> u32 {
        self.config.recovery_success_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prober_recovery() {
        let config = super::super::config::ProbingConfig {
            enabled: true,
            interval_healthy_sec: 30,
            interval_unstable_sec: 5,
            probe_timeout_ms: 3000,
            recovery_success_count: 2,
        };
        let prober = Prober::new(config);

        prober.record_result("test-model", &ProbeResult::Success { latency_ms: 100 });
        assert!(!prober.is_recovered("test-model"));
        prober.record_result("test-model", &ProbeResult::Success { latency_ms: 100 });
        assert!(prober.is_recovered("test-model"));

        // 失败后重置
        prober.record_result("test-model", &ProbeResult::Failure { error: "timeout".to_string() });
        assert!(!prober.is_recovered("test-model"));
    }
}