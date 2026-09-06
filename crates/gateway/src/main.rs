mod middleware;
mod routes;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::routes::metrics::MetricsCollector;
use llm_smart_router_circuit_breaker::config::Config as BreakerConfig;
use llm_smart_router_circuit_breaker::CircuitBreaker;
use llm_smart_router_provider::registry::ProviderRegistry;
use llm_smart_router_router_engine::pipeline::RouterEngine;
use llm_smart_router_storage::cache::Cache;
use llm_smart_router_storage::duckdb::AnalyticsDB;
use llm_smart_router_storage::json_store::JsonStore;

/// 应用共享状态
pub struct AppState {
    pub router: RouterEngine,
    pub cache: Cache,
    pub analytics: AnalyticsDB,
    pub config: JsonStore,
    pub api_keys: Vec<String>,
    pub metrics: MetricsCollector,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // 初始化存储
    let cache = Cache::new(10000, 300);
    let analytics = AnalyticsDB::new("data/smart_router.duckdb")?;
    let config = JsonStore::new("data/config/")?;

    // 加载提供商配置
    let registry = Arc::new(ProviderRegistry::new());
    if let Ok(providers) = config.load_providers() {
        for p in providers {
            let model_ids: Vec<String> = p.models.iter().map(|m| m.id.clone()).collect();
            let is_openai = p.name.eq_ignore_ascii_case("openai");
            let is_anthropic = p.name.eq_ignore_ascii_case("anthropic");

            let adapter: Arc<dyn llm_smart_router_provider::registry::ProviderAdapter> =
                if is_openai {
                    llm_smart_router_provider::adapters::create_openai(
                        p.api_key,
                        Some(p.api_base_url),
                        model_ids.clone(),
                    )
                } else if is_anthropic {
                    llm_smart_router_provider::adapters::create_anthropic(
                        p.api_key,
                        Some(p.api_base_url),
                        model_ids.clone(),
                    )
                } else {
                    tracing::warn!("unknown provider type: {}, defaulting to OpenAI", p.name);
                    llm_smart_router_provider::adapters::create_openai(
                        p.api_key,
                        Some(p.api_base_url),
                        model_ids.clone(),
                    )
                };
            registry.register(&p.name, adapter);
            tracing::info!(
                "registered provider: {} with {} models",
                p.name,
                model_ids.len()
            );
        }
    }

    // 初始化熔断器
    let breaker = Arc::new(CircuitBreaker::new(BreakerConfig::default()));

    // 初始化路由引擎
    let router = RouterEngine::new(breaker, registry);

    // 创建共享状态
    let metrics = MetricsCollector::new();
    let state = Arc::new(AppState {
        router,
        cache,
        analytics,
        config,
        api_keys: vec!["sk-local-dev".to_string()],
        metrics,
    });

    // 构建路由
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(routes::chat_completions::handler),
        )
        .route("/v1/messages", post(routes::messages::handler))
        .route("/v1/responses", post(routes::responses::handler))
        .route("/v1/models", get(routes::models::handler))
        .route("/health", get(routes::health::handler))
        .route("/health/metrics", get(routes::metrics::metrics_handler))
        .route(
            "/health/detail",
            get(routes::metrics::health_detail_handler),
        )
        .route("/admin/providers", post(routes::admin::register_provider))
        .route("/admin/providers", get(routes::admin::list_providers))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 启动服务
    let addr = "0.0.0.0:8080";
    tracing::info!("Starting LLM Smart Router on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
