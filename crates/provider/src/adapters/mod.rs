pub mod openai;
pub mod anthropic;

use std::sync::Arc;
use super::registry::ProviderAdapter;

/// 创建 OpenAI 适配器
pub fn create_openai(api_key: String, base_url: Option<String>, models: Vec<String>) -> Arc<dyn ProviderAdapter> {
    Arc::new(openai::OpenAIAdapter::new(api_key, base_url, models))
}

/// 创建 Anthropic 适配器
pub fn create_anthropic(api_key: String, base_url: Option<String>, models: Vec<String>) -> Arc<dyn ProviderAdapter> {
    Arc::new(anthropic::AnthropicAdapter::new(api_key, base_url, models))
}