use super::state::ErrorType;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// 时间桶
#[derive(Debug, Clone)]
pub struct TimeBucket {
    pub timestamp: DateTime<Utc>,
    pub success_count: u64,
    pub failure_count: u64,
    pub total_latency: u64,
    pub latency_samples: Vec<u64>,
    pub error_types: Vec<ErrorType>,
}

/// 滑动窗口统计
pub struct SlidingWindow {
    buckets: VecDeque<TimeBucket>,
    window_size_secs: u64,
    bucket_size_secs: u64,
    max_buckets: usize,
}

/// 窗口统计结果
#[derive(Debug, Clone)]
pub struct WindowStats {
    pub total_requests: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub error_rate: f64,
    pub p50_latency: u64,
    pub p95_latency: u64,
    pub avg_latency: u64,
    pub error_type_counts: std::collections::HashMap<String, u64>,
}

impl SlidingWindow {
    pub fn new(window_size_secs: u64, max_buckets: usize) -> Self {
        let bucket_size_secs = window_size_secs / max_buckets as u64;
        Self {
            buckets: VecDeque::new(),
            window_size_secs,
            bucket_size_secs,
            max_buckets,
        }
    }

    fn current_bucket(&mut self) -> &mut TimeBucket {
        let now = Utc::now();
        let bucket_time = self.bucket_time(now);

        if let Some(last) = self.buckets.back() {
            if last.timestamp == bucket_time {
                return self.buckets.back_mut().unwrap();
            }
        }

        // 清理过期桶
        while self.buckets.len() >= self.max_buckets {
            self.buckets.pop_front();
        }

        self.buckets.push_back(TimeBucket {
            timestamp: bucket_time,
            success_count: 0,
            failure_count: 0,
            total_latency: 0,
            latency_samples: Vec::new(),
            error_types: Vec::new(),
        });
        self.buckets.back_mut().unwrap()
    }

    fn bucket_time(&self, time: DateTime<Utc>) -> DateTime<Utc> {
        let secs = time.timestamp();
        let bucket = secs - (secs % self.bucket_size_secs as i64);
        DateTime::from_timestamp(bucket, 0).unwrap()
    }

    pub fn record_success(&mut self, latency_ms: u64) {
        let bucket = self.current_bucket();
        bucket.success_count += 1;
        bucket.total_latency += latency_ms;
        bucket.latency_samples.push(latency_ms);
    }

    pub fn record_failure(&mut self, error_type: ErrorType) {
        let bucket = self.current_bucket();
        bucket.failure_count += 1;
        bucket.error_types.push(error_type);
    }

    pub fn stats(&self) -> WindowStats {
        let mut total_requests = 0u64;
        let mut success_count = 0u64;
        let mut failure_count = 0u64;
        let mut all_latencies: Vec<u64> = Vec::new();
        let mut error_type_counts: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();

        for bucket in &self.buckets {
            total_requests += bucket.success_count + bucket.failure_count;
            success_count += bucket.success_count;
            failure_count += bucket.failure_count;
            all_latencies.extend(&bucket.latency_samples);
            for err in &bucket.error_types {
                let key = format!("{:?}", err);
                *error_type_counts.entry(key).or_insert(0) += 1;
            }
        }

        let error_rate = if total_requests > 0 {
            failure_count as f64 / total_requests as f64
        } else {
            0.0
        };

        all_latencies.sort();
        let len = all_latencies.len();
        let p50 = if len > 0 { all_latencies[len / 2] } else { 0 };
        let p95 = if len > 0 {
            let idx = (len as f64 * 0.95) as usize;
            all_latencies[idx.min(len - 1)]
        } else {
            0
        };
        let avg_latency = if len > 0 {
            all_latencies.iter().sum::<u64>() / len as u64
        } else {
            0
        };

        WindowStats {
            total_requests,
            success_count,
            failure_count,
            error_rate,
            p50_latency: p50,
            p95_latency: p95,
            avg_latency,
            error_type_counts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window() {
        let mut sw = SlidingWindow::new(60, 6);
        sw.record_success(100);
        sw.record_success(200);
        sw.record_failure(ErrorType::Timeout);

        let stats = sw.stats();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 1);
        assert!((stats.error_rate - 0.333).abs() < 0.01);
    }
}
