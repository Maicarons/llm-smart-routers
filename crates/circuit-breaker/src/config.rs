use serde::{Deserialize, Serialize};

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub sliding_window: SlidingWindowConfig,
    pub error_thresholds: ErrorThresholds,
    pub latency_trend: LatencyTrendConfig,
    pub scoring: super::health_scorer::HealthScoreConfig,
    pub probing: ProbingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sliding_window: SlidingWindowConfig::default(),
            error_thresholds: ErrorThresholds::default(),
            latency_trend: LatencyTrendConfig::default(),
            scoring: Default::default(),
            probing: ProbingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidingWindowConfig {
    pub size_seconds: u64,
    pub bucket_count: usize,
}

impl Default for SlidingWindowConfig {
    fn default() -> Self {
        Self {
            size_seconds: 120,
            bucket_count: 12,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorThresholds {
    pub timeout_trigger: f64,
    pub rate_limit_trigger: f64,
    pub server_error_trigger: f64,
    pub auth_error_trigger: f64,
}

impl Default for ErrorThresholds {
    fn default() -> Self {
        Self {
            timeout_trigger: 0.15,
            rate_limit_trigger: 0.10,
            server_error_trigger: 0.05,
            auth_error_trigger: 0.01,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyTrendConfig {
    pub ewma_alpha: f64,
    pub acceleration_windows: u32,
    pub p95_threshold_ms: u64,
}

impl Default for LatencyTrendConfig {
    fn default() -> Self {
        Self {
            ewma_alpha: 0.3,
            acceleration_windows: 3,
            p95_threshold_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbingConfig {
    pub enabled: bool,
    pub interval_healthy_sec: u64,
    pub interval_unstable_sec: u64,
    pub probe_timeout_ms: u64,
    pub recovery_success_count: u32,
}

impl Default for ProbingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_healthy_sec: 30,
            interval_unstable_sec: 5,
            probe_timeout_ms: 3000,
            recovery_success_count: 2,
        }
    }
}