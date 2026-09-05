use async_trait::async_trait;
use llm_smart_router_protocol::converter::{OpenAI, Converter};
use llm_smart_router_protocol::openai;
use llm_smart_router_protocol::*;
use super::super::registry::ProviderAdapter;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAIAdapter {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl OpenAIAdapter {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        }
    }
}

#[async_trait]
impl ProviderAdapter for OpenAIAdapter {
    fn name(&self) -> &str {
        "openai"
    }

    fn models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]
    }

    async fn chat(&self, request: &UnifiedRequest) -> anyhow::Result<UnifiedResponse> {
        let converter = OpenAI;
        let openai_req = converter.from_unified_response(&UnifiedResponse {
            id: "".to_string(),
            model: request.model.clone(),
            content: "".to_string(),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage::default(),
            tool_calls: vec![],
        })?;

        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(openai_req)
            .send()
            .await?;

        let body = response.bytes().await?;
        let chat_resp: openai::ChatCompletionResponse = serde_json::from_slice(&body)?;

        let finish_reason = match chat_resp.choices.first()
            .and_then(|c| c.finish_reason.as_deref())
        {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolCalls,
            Some("content_filter") => FinishReason::ContentFilter,
            _ => FinishReason::Error,
        };

        let content = chat_resp.choices.first()
            .and_then(|c| c.message.content.as_deref())
            .unwrap_or("")
            .to_string();

        Ok(UnifiedResponse {
            id: chat_resp.id,
            model: chat_resp.model,
            content,
            finish_reason,
            usage: TokenUsage {
                prompt_tokens: chat_resp.usage.prompt_tokens,
                completion_tokens: chat_resp.usage.completion_tokens,
                total_tokens: chat_resp.usage.total_tokens,
            },
            tool_calls: vec![],
        })
    }
}