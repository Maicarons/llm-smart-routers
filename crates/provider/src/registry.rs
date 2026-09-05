use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use llm_smart_router_protocol::*;

/// 提供商适配器 trait
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, request: &UnifiedRequest) -> anyhow::Result<UnifiedResponse>;
    fn models(&self) -> Vec<String>;
}

/// 模型信息
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub capabilities: Vec<String>,
}

/// 提供商注册中心
pub struct ProviderRegistry {
    providers: DashMap<String, Arc<dyn ProviderAdapter>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
        }
    }

    pub fn register(&self, name: &str, adapter: Arc<dyn ProviderAdapter>) {
        self.providers.insert(name.to_string(), adapter);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn ProviderAdapter>> {
        self.providers.get(name).map(|entry| entry.value().clone())
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        let mut models = Vec::new();
        for entry in self.providers.iter() {
            let provider = entry.key();
            let adapter = entry.value();
            for model_id in adapter.models() {
                models.push(ModelInfo {
                    id: model_id,
                    provider: provider.clone(),
                    capabilities: vec!["chat".to_string()],
                });
            }
        }
        models
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}