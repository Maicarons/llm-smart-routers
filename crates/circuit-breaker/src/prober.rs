/// 主动健康探测
pub struct Prober {
    config: super::config::ProbingConfig,
}

impl Prober {
    pub fn new(config: super::config::ProbingConfig) -> Self {
        Self { config }
    }

    /// 获取探测间隔
    pub fn interval(&self, is_healthy: bool) -> std::time::Duration {
        if is_healthy {
            std::time::Duration::from_secs(self.config.interval_healthy_sec)
        } else {
            std::time::Duration::from_secs(self.config.interval_unstable_sec)
        }
    }

    /// 探测超时
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.config.probe_timeout_ms)
    }

    /// 恢复所需成功次数
    pub fn recovery_count(&self) -> u32 {
        self.config.recovery_success_count
    }
}