use crate::models::*;
use async_trait::async_trait;
use llm_smart_router_provider::registry::ModelInfo;
use std::collections::HashMap;
use std::sync::Arc;

/// 策略插件 trait - 允许用户自定义路由策略
#[async_trait]
pub trait StrategyPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn select(
        &self,
        models: &[ModelInfo],
        context: &RouteContext,
    ) -> anyhow::Result<RouteDecision>;
}

/// 策略插件注册表
pub struct StrategyPluginRegistry {
    plugins: HashMap<String, Arc<dyn StrategyPlugin>>,
}

impl StrategyPluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: Arc<dyn StrategyPlugin>) {
        tracing::info!(
            "registering strategy plugin: {} ({})",
            plugin.name(),
            plugin.description()
        );
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    /// 获取插件
    pub fn get(&self, name: &str) -> Option<Arc<dyn StrategyPlugin>> {
        self.plugins.get(name).cloned()
    }

    /// 列出所有已注册插件
    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// 插件数量
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for StrategyPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    #[async_trait]
    impl StrategyPlugin for TestPlugin {
        fn name(&self) -> &str {
            "test"
        }
        fn description(&self) -> &str {
            "test plugin"
        }
        async fn select(
            &self,
            models: &[ModelInfo],
            _context: &RouteContext,
        ) -> anyhow::Result<RouteDecision> {
            models
                .first()
                .map(|m| RouteDecision {
                    provider: m.provider.clone(),
                    model: m.id.clone(),
                    confidence: 1.0,
                    task_type: None,
                })
                .ok_or_else(|| anyhow::anyhow!("no models"))
        }
    }

    #[tokio::test]
    async fn test_plugin_registry() {
        let mut registry = StrategyPluginRegistry::new();
        registry.register(Arc::new(TestPlugin));
        assert_eq!(registry.count(), 1);
        assert!(registry.get("test").is_some());
    }
}
