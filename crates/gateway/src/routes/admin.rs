use std::sync::Arc;
use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::{json, Value};
use llm_smart_router_storage::json_store::ProviderConfig;
use crate::AppState;

/// 注册提供商
pub async fn register_provider(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let config: ProviderConfig = serde_json::from_slice(&body).map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(json!({"error": format!("invalid config: {}", e)})))
    })?;

    let model_ids: Vec<String> = config.models.iter().map(|m| m.id.clone()).collect();
    let adapter = match config.name.as_str() {
        "openai" | "OpenAI" => {
            llm_smart_router_provider::adapters::create_openai(
                config.api_key.clone(),
                Some(config.api_base_url.clone()),
                model_ids,
            )
        }
        "anthropic" | "Anthropic" => {
            llm_smart_router_provider::adapters::create_anthropic(
                config.api_key.clone(),
                Some(config.api_base_url.clone()),
                model_ids,
            )
        }
        _ => {
            return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "unsupported provider type"}))));
        }
    };

    state.router.registry.register(&config.name, adapter);
    tracing::info!("registered provider: {}", config.name);

    // 保存到 JSON 配置
    let mut providers = state.config.load_providers().unwrap_or_default();
    providers.push(config);
    let _ = state.config.save_providers(&providers);

    Ok(Json(json!({"status": "ok", "message": "provider registered"})))
}

/// 列出提供商
pub async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let providers = state.config.load_providers().unwrap_or_default();
    Json(json!({
        "providers": providers.iter().map(|p| json!({
            "name": p.name,
            "api_base_url": p.api_base_url,
            "models": p.models.iter().map(|m| json!({
                "id": m.id,
                "capabilities": m.capabilities
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    }))
}