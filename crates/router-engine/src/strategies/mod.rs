pub mod failover;
pub mod load_balance;
pub mod manual;
pub mod plugin;
pub mod task_aware;

use super::models::*;
use async_trait::async_trait;
use llm_smart_router_provider::registry::ModelInfo;

/// 路由策略 trait
#[async_trait]
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    async fn select(
        &self,
        models: &[ModelInfo],
        context: &RouteContext,
    ) -> anyhow::Result<RouteDecision>;
}
