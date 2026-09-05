pub mod state;
pub mod sliding_window;
pub mod health_scorer;
pub mod prober;
pub mod config;

use std::sync::Arc;
use dashmap::DashMap;
use state::*;
use sliding_window::*;
use health_scorer::*;
use config::*;

/// 智能熔断器
pub struct CircuitBreaker {
    states: DashMap<String, BreakerStatus>,
    windows: DashMap<String, SlidingWindow>,
    scorer: HealthScorer,
    config: Config,
}

impl CircuitBreaker {
    pub fn new(config: Config) -> Self {
        Self {
            states: DashMap::new(),
            windows: DashMap::new(),
            scorer: HealthScorer::new(config.scoring.clone()),
            config,
        }
    }

    /// 检查指定模型是否可用
    pub fn check(&self, model: &str) -> Result<(), BreakerError> {
        if let Some(status) = self.states.get(model) {
            match status.state {
                BreakerState::Closed | BreakerState::Warning => Ok(()),
                BreakerState::Degraded => Err(BreakerError::Degraded(status.health_score)),
                BreakerState::Limited => Err(BreakerError::Limited(status.health_score)),
                BreakerState::Open => Err(BreakerError::Open),
            }
        } else {
            Ok(()) // 新模型默认可用
        }
    }

    /// 记录一次调用结果
    pub fn record(&self, model: &str, success: bool, latency_ms: u64, error_type: Option<ErrorType>) {
        let mut window = self.windows.entry(model.to_string())
            .or_insert_with(|| SlidingWindow::new(
                self.config.sliding_window.size_seconds,
                self.config.sliding_window.bucket_count,
            ));
        
        if success {
            window.record_success(latency_ms);
        } else {
            window.record_failure(error_type.unwrap_or(ErrorType::ServerError));
        }

        let stats = window.stats();
        let health = self.scorer.score(&stats);

        let (state, _) = BreakerState::from_health(health, &self.config);
        let mut status = self.states.entry(model.to_string()).or_insert_with(|| BreakerStatus {
            state: BreakerState::Closed,
            health_score: 1.0,
            last_failure: None,
            failure_count: 0,
            total_requests: 0,
        });

        status.state = state;
        status.health_score = health;
        status.total_requests += 1;
        if !success {
            status.failure_count += 1;
            status.last_failure = Some(std::time::Instant::now());
        }
    }

    /// 获取模型健康度
    pub fn health_score(&self, model: &str) -> Option<f64> {
        self.states.get(model).map(|s| s.health_score)
    }
}

#[derive(Debug, Clone)]
pub enum BreakerError {
    Degraded(f64),
    Limited(f64),
    Open,
}

impl std::fmt::Display for BreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakerError::Degraded(s) => write!(f, "service degraded (health: {})", s),
            BreakerError::Limited(s) => write!(f, "service limited (health: {})", s),
            BreakerError::Open => write!(f, "circuit breaker open"),
        }
    }
}

impl std::error::Error for BreakerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaker_initial_state() {
        let cb = CircuitBreaker::new(Config::default());
        assert!(cb.check("test-model").is_ok());
    }

    #[test]
    fn test_breaker_opens_after_failures() {
        let cb = CircuitBreaker::new(Config::default());
        for _ in 0..10 {
            cb.record("test-model", false, 1000, Some(ErrorType::Timeout));
        }
        // 持续失败，熔断器应打开或限制
        let result = cb.check("test-model");
        assert!(result.is_err() || cb.health_score("test-model").unwrap() < 0.5,
            "breaker should degrade after failures, health: {:?}", cb.health_score("test-model"));
    }

    #[test]
    fn test_breaker_stays_closed_on_success() {
        let cb = CircuitBreaker::new(Config::default());
        for _ in 0..20 {
            cb.record("test-model", true, 100, None);
        }
        assert!(cb.check("test-model").is_ok());
        // 持续成功应该保持高健康度
        assert!(cb.health_score("test-model").unwrap() > 0.5, "health: {}", cb.health_score("test-model").unwrap());
    }
}