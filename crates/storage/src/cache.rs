use std::time::Duration;
use moka::future::Cache as MokaCache;

/// 缓存层 (moka + dashmap)
pub struct Cache {
    response_cache: MokaCache<String, Vec<u8>>,
    rate_limits: dashmap::DashMap<String, (u64, std::time::Instant)>,
    breaker_states: dashmap::DashMap<String, f64>,
}

impl Cache {
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        Self {
            response_cache: MokaCache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(ttl_secs))
                .build(),
            rate_limits: dashmap::DashMap::new(),
            breaker_states: dashmap::DashMap::new(),
        }
    }

    /// 获取缓存
    pub async fn get_cached(&self, key: &str) -> Option<Vec<u8>> {
        self.response_cache.get(key).await
    }

    /// 设置缓存
    pub async fn set_cache(&self, key: &str, value: Vec<u8>) {
        self.response_cache.insert(key.to_string(), value).await;
    }

    /// 速率限制检查 (返回是否允许通过)
    pub fn check_rate_limit(&self, key: &str, limit: u64, window_secs: u64) -> bool {
        let now = std::time::Instant::now();
        let mut entry = self.rate_limits.entry(key.to_string()).or_insert((0, now));
        let (count, start) = entry.value_mut();

        if now.duration_since(*start).as_secs() > window_secs {
            *count = 0;
            *start = now;
        }

        *count += 1;
        *count <= limit
    }

    /// 获取熔断器状态
    pub fn get_breaker(&self, key: &str) -> Option<f64> {
        self.breaker_states.get(key).map(|v| *v.value())
    }

    /// 设置熔断器状态
    pub fn set_breaker(&self, key: &str, health: f64) {
        self.breaker_states.insert(key.to_string(), health);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic() {
        let cache = Cache::new(100, 60);
        cache.set_cache("test", b"hello".to_vec()).await;
        let result = cache.get_cached("test").await;
        assert_eq!(result, Some(b"hello".to_vec()));
    }

    #[test]
    fn test_rate_limit() {
        let cache = Cache::new(100, 60);
        // 5 次限额
        for _ in 0..5 {
            assert!(cache.check_rate_limit("test-key", 5, 60));
        }
        // 第 6 次应被限
        assert!(!cache.check_rate_limit("test-key", 5, 60));
    }
}