use super::super::registry::ProviderAdapter;
use async_trait::async_trait;
use llm_smart_router_protocol::anthropic;
use llm_smart_router_protocol::*;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";

pub struct AnthropicAdapter {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model_list: Vec<String>,
}

impl AnthropicAdapter {
    pub fn new(api_key: String, base_url: Option<String>, models: Vec<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model_list: if models.is_empty() {
                vec![
                    "claude-3-5-sonnet".to_string(),
                    "claude-3-opus".to_string(),
                    "claude-3-sonnet".to_string(),
                    "claude-3-haiku".to_string(),
                ]
            } else {
                models
            },
        }
    }

    fn build_request(&self, request: &UnifiedRequest) -> anthropic::MessagesRequest {
        let mut messages = Vec::new();
        for msg in &request.messages {
            let role = match msg.role {
                UnifiedRole::User => "user",
                UnifiedRole::Assistant => "assistant",
                _ => "user",
            };
            let content = match &msg.content {
                UnifiedContent::Text(t) => anthropic::AnthropicContent::Text(t.clone()),
                UnifiedContent::Parts(_) => anthropic::AnthropicContent::Text("".to_string()),
            };
            messages.push(anthropic::AnthropicMessage {
                role: role.to_string(),
                content,
            });
        }

        anthropic::MessagesRequest {
            model: request.model.clone(),
            messages,
            system: request.system_prompt.clone(),
            max_tokens: request.max_tokens.unwrap_or(1024),
            metadata: None,
            stop_sequences: if request.stop_sequences.is_empty() {
                None
            } else {
                Some(request.stop_sequences.clone())
            },
            stream: Some(request.stream),
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: None,
            tools: None,
            tool_choice: None,
        }
    }
}

#[async_trait]
impl ProviderAdapter for AnthropicAdapter {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn models(&self) -> Vec<String> {
        self.model_list.clone()
    }

    async fn chat(&self, request: &UnifiedRequest) -> anyhow::Result<UnifiedResponse> {
        let anthropic_req = self.build_request(request);
        let url = format!("{}/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_req)
            .send()
            .await?;

        let status = response.status();
        let body = response.bytes().await?;

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&body);
            return Err(anyhow::anyhow!("API error ({}): {}", status, error_text));
        }

        let msg_resp: anthropic::MessagesResponse = serde_json::from_slice(&body).map_err(|e| {
            anyhow::anyhow!(
                "failed to parse response: {} - body: {}",
                e,
                String::from_utf8_lossy(&body)
            )
        })?;

        let finish_reason = match msg_resp.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("tool_use") => FinishReason::ToolCalls,
            _ => FinishReason::Error,
        };

        let content = msg_resp
            .content
            .iter()
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
