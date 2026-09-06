use crate::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use llm_smart_router_protocol::*;
use llm_smart_router_router_engine::models::RouteContext;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 协议转换：Anthropic → Unified
    let converter = AnthropicMessages;
    let unified = converter.to_unified_request(&body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("invalid request: {}", e)})),
        )
    })?;

    // 路由决策
    let context = RouteContext {
        model_hint: unified.model.clone(),
        strategy_name: None,
        max_tokens: unified.max_tokens,
        temperature: unified.temperature,
        task_type: None,
    };

    let decision = state.router.route(&unified, &context).await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("routing failed: {}", e)})),
        )
    })?;

    // 获取提供商适配器
    let adapter = state
        .router
        .registry
        .get(&decision.provider)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "provider not found"})),
            )
        })?;

    // 调用提供商
    let response = adapter.chat(&unified).await.map_err(|e| {
        state.router.breaker.record(&decision.model, false, 0, None);
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": format!("provider error: {}", e)})),
        )
    })?;

    // 记录熔断器
    state.router.breaker.record(&decision.model, true, 0, None);

    // 记录调用日志
    let log = llm_smart_router_storage::duckdb::CallLog {
        id: uuid::Uuid::new_v4().to_string(),
        provider: decision.provider,
        model: decision.model,
        task_type: None,
        input_tokens: response.usage.prompt_tokens,
        output_tokens: response.usage.completion_tokens,
        latency_ms: 0,
        is_success: true,
        error_type: None,
        cost_usd: 0.0,
        created_at: chrono::Utc::now(),
    };
    let _ = state.analytics.log_call(&log);

    // 协议转换：Unified → Anthropic
    let output = converter.from_unified_response(&response).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("conversion error: {}", e)})),
        )
    })?;

    let value: Value = serde_json::from_slice(&output).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("json error: {}", e)})),
        )
    })?;

    Ok(Json(value))
}
