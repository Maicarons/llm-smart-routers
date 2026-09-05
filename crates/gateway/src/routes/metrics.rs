use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::{json, Value};
use chrono::Utc;
use crate::AppState;

/// 指标收集器 - 与应用逻辑隔离，仅通过原子操作更新
pub struct MetricsCollector {
    // 请求统计
    pub total_requests: AtomicU64,
    pub success_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_tokens: AtomicU64,
    // 按提供商统计
    provider_requests: dashmap::DashMap<String, AtomicU64>,
    provider_errors: dashmap::DashMap<String, AtomicU64>,
    provider_tokens: dashmap::DashMap<String, AtomicU64>,
    start_time: chrono::DateTime<Utc>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            success_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            provider_requests: dashmap::DashMap::new(),
            provider_errors: dashmap::DashMap::new(),
            provider_tokens: dashmap::DashMap::new(),
            start_time: Utc::now(),
        }
    }

    pub fn record_request(&self, provider: &str, success: bool, tokens: u32) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if success {
            self.success_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        self.total_tokens.fetch_add(tokens as u64, Ordering::Relaxed);

        let p = provider.to_string();
        self.provider_requests.entry(p.clone()).or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
        if !success {
            self.provider_errors.entry(p.clone()).or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }
        self.provider_tokens.entry(p.clone()).or_insert_with(|| AtomicU64::new(0))
            .fetch_add(tokens as u64, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let uptime = (Utc::now() - self.start_time).num_seconds();
        let mut provider_stats = Vec::new();
        for entry in self.provider_requests.iter() {
            let provider = entry.key().clone();
            let reqs = entry.value().load(Ordering::Relaxed);
            let errs = self.provider_errors.get(&provider)
                .map(|e| e.load(Ordering::Relaxed))
                .unwrap_or(0);
            let tokens = self.provider_tokens.get(&provider)
                .map(|t| t.load(Ordering::Relaxed))
                .unwrap_or(0);
            provider_stats.push(ProviderMetrics {
                provider,
                requests: reqs,
                errors: errs,
                tokens,
            });
        }
        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            success_requests: self.success_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            total_tokens: self.total_tokens.load(Ordering::Relaxed),
            uptime_seconds: uptime as u64,
            provider_stats,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub success_requests: u64,
    pub failed_requests: u64,
    pub total_tokens: u64,
    pub uptime_seconds: u64,
    pub provider_stats: Vec<ProviderMetrics>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProviderMetrics {
    pub provider: String,
    pub requests: u64,
    pub errors: u64,
    pub tokens: u64,
}

/// Prometheus 格式的指标端点
pub async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snapshot = state.metrics.snapshot();
    let mut output = String::new();

    output.push_str("# HELP llm_smart_router_total_requests Total requests\n");
    output.push_str("# TYPE llm_smart_router_total_requests counter\n");
    output.push_str(&format!("llm_smart_router_total_requests {} {}\n", snapshot.total_requests, Utc::now().timestamp()));

    output.push_str("# HELP llm_smart_router_success_requests Successful requests\n");
    output.push_str(&format!("llm_smart_router_success_requests {}\n", snapshot.success_requests));

    output.push_str("# HELP llm_smart_router_failed_requests Failed requests\n");
    output.push_str(&format!("llm_smart_router_failed_requests {}\n", snapshot.failed_requests));

    output.push_str("# HELP llm_smart_router_total_tokens Total tokens processed\n");
    output.push_str(&format!("llm_smart_router_total_tokens {}\n", snapshot.total_tokens));

    output.push_str("# HELP llm_smart_router_uptime_seconds Uptime in seconds\n");
    output.push_str(&format!("llm_smart_router_uptime_seconds {}\n", snapshot.uptime_seconds));

    for ps in &snapshot.provider_stats {
        output.push_str(&format!("llm_smart_router_provider_requests{{provider=\"{}\"}} {}\n", ps.provider, ps.requests));
        output.push_str(&format!("llm_smart_router_provider_errors{{provider=\"{}\"}} {}\n", ps.provider, ps.errors));
        output.push_str(&format!("llm_smart_router_provider_tokens{{provider=\"{}\"}} {}\n", ps.provider, ps.tokens));
    }

    axum::response::Response::new(
        axum::body::Body::from(output)
    )
}

/// JSON 格式的健康详情
pub async fn health_detail_handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    let snapshot = state.metrics.snapshot();
    Json(json!({
        "status": "ok",
        "version": "0.1.0",
        "service": "llm-smart-router",
        "uptime_seconds": snapshot.uptime_seconds,
        "metrics": {
            "total_requests": snapshot.total_requests,
            "success_requests": snapshot.success_requests,
            "failed_requests": snapshot.failed_requests,
            "total_tokens": snapshot.total_tokens,
            "providers": snapshot.provider_stats.iter().map(|p| json!({
                "name": p.provider,
                "requests": p.requests,
                "errors": p.errors,
                "tokens": p.tokens,
            })).collect::<Vec<_>>()
        }
    }))
}