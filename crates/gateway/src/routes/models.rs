use std::sync::Arc;
use axum::{
    extract::State,
    response::Json,
};
use serde_json::{json, Value};
use crate::AppState;

pub async fn handler(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let models = state.router.registry.list_models();
    Json(json!({
        "object": "list",
        "data": models.iter().map(|m| json!({
            "id": m.id,
            "provider": m.provider,
            "capabilities": m.capabilities,
            "object": "model"
        })).collect::<Vec<_>>()
    }))
}