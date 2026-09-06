use crate::AppState;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::sync::Arc;

/// API Key 认证中间件
pub async fn auth_middleware(
    state: axum::extract::State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 健康检查和管理接口不需要认证
    let path = req.uri().path().to_string();
    if path == "/health" || path.starts_with("/health/") || path.starts_with("/admin/") {
        return Ok(next.run(req).await);
    }

    // 提取 API Key 并检查
    let api_key = extract_api_key(&req);
    match api_key {
        Some(key) if state.api_keys.contains(&key) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

fn extract_api_key(req: &Request) -> Option<String> {
    // 尝试从 Authorization 头获取
    if let Some(value) = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(value.to_string());
    }
    // 尝试从 x-api-key 头获取 (Anthropic 兼容)
    if let Some(value) = req.headers().get("x-api-key").and_then(|v| v.to_str().ok()) {
        return Some(value.to_string());
    }
    None
}
