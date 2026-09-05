use async_trait::async_trait;
use llm_smart_router_protocol::converter::{AnthropicMessages, Converter};
use llm_smart_router_protocol::anthropic;
use llm_smart_router_protocol::*;
use super::super::registry::ProviderAdapter;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";

pub struct AnthropicAdapter {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl AnthropicAdapter {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        }
    }
}

#[async_trait]
impl ProviderAdapter for AnthropicAdapter {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn models(&self) -> Vec<String> {
        vec![
            "claude-3-5-sonnet".to_string(),
            "claude-3-opus".to_string(),
            "claude-3-sonnet".to_string(),
            "claude-3-haiku".to_string(),
        ]
    }

    async fn chat(&self, request: &UnifiedRequest) -> anyhow::Result<UnifiedResponse> {
        let converter = AnthropicMessages;
        let anthropic_req = converter.from_unified_response(&UnifiedResponse {
            id: "".to_string(),
            model: request.model.clone(),
            content: "".to_string(),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage::default(),
            tool_calls: vec![],
        })?;

        let url = format!("{}/messages", self.base_url);
        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .body(anthropic_req)
            .send()
            .await?;

        let body = response.bytes().await?;
        let msg_resp: anthropic::MessagesResponse = serde_json::from_slice(&body)?;

        let finish_reason = match msg_resp.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("tool_use") => FinishReason::ToolCalls,
            _ => FinishReason::Error,
        };

        let content = msg_resp.content.iter()
            .filter_map(|block| {
                if let anthropic::ContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        Ok(UnifiedResponse {
            id: msg_resp.id,
            model: msg_resp.model,
            content,
            finish_reason,
            usage: TokenUsage {
                prompt_tokens: msg_resp.usage.input_tokens,
                completion_tokens: msg_resp.usage.output_tokens,
                total_tokens: msg_resp.usage.input_tokens + msg_resp.usage.output_tokens,
            },
            tool_calls: vec![],
        })
    }
}