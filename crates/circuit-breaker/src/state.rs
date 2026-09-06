use super::config::Config;
use serde::{Deserialize, Serialize};

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BreakerState {
    Closed,   // 全流量 (health 0.8-1.0)
    Warning,  // 预警 (health 0.6-0.8)
    Degraded, // 降级 (health 0.3-0.6)
    Limited,  // 限制 (health 0.1-0.3)
    Open,     // 熔断 (health < 0.1)
}

impl BreakerState {
    pub fn from_health(health: f64, config: &Config) -> (Self, f64) {
        let thresholds = &config.error_thresholds;
        if health >= 0.8 {
            (BreakerState::Closed, 1.0)
        } else if health >= 0.6 {
            (BreakerState::Warning, 0.8)
        } else if health >= 0.3 {
            (BreakerState::Degraded, 0.5)
        } else if health >= 0.1 {
            (BreakerState::Limited, 0.2)
        } else {
            (BreakerState::Open, 0.0)
        }
    }
}

/// 熔断器状态信息
#[derive(Debug, Clone)]
pub struct BreakerStatus {
    pub state: BreakerState,
    pub health_score: f64,
    pub last_failure: Option<std::time::Instant>,
    pub failure_count: u64,
    pub total_requests: u64,
}

/// 错误类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorType {
    Timeout,
    RateLimit,
    ServerError,
    AuthError,
    InvalidResponse,
}
