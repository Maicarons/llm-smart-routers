use async_trait::async_trait;
use llm_smart_router_protocol::openai;
use llm_smart_router_protocol::*;
use super::super::registry::ProviderAdapter;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAIAdapter {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model_list: Vec<String>,
}

impl OpenAIAdapter {
    pub fn new(api_key: String, base_url: Option<String>, models: Vec<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model_list: if models.is_empty() {
                vec![
                    "gpt-4o".to_string(),
                    "gpt-4o-mini".to_string(),
                    "gpt-4-turbo".to_string(),
                    "gpt-3.5-turbo".to_string(),
                ]
            } else {
                models
            },
        }
    }

    fn build_request(&self, request: &UnifiedRequest) -> openai::ChatCompletionRequest {
        let mut messages = Vec::new();

        // 添加 system prompt
        if let Some(system) = &request.system_prompt {
            messages.push(openai::ChatMessage {
                role: "system".to_string(),
                content: serde_json::Value::String(system.clone()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // 添加对话消息
        for msg in &request.messages {
            let role = match msg.role {
                UnifiedRole::System => "system",
                UnifiedRole::User => "user",
                UnifiedRole::Assistant => "assistant",
                UnifiedRole::Tool => "tool",
            };
            let content = match &msg.content {
                UnifiedContent::Text(t) => serde_json::Value::String(t.clone()),
                UnifiedContent::Parts(_) => serde_json::Value::String("".to_string()),
            };
            messages.push(openai::ChatMessage {
                role: role.to_string(),
                content,
                name: msg.name.clone(),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        openai::ChatCompletionRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(request.stream),
            stop: if request.stop_sequences.is_empty() { None } else { Some(request.stop_sequences.clone()) },
            tools: None,
            tool_choice: None,
            user: None,
        }
    }
}

#[async_trait]
impl ProviderAdapter for OpenAIAdapter {
    fn name(&self) -> &str {
        "openai"
    }

    fn models(&self) -> Vec<String> {
        self.model_list.clone()
    }

    async fn chat(&self, request: &UnifiedRequest) -> anyhow::Result<UnifiedResponse> {
        let openai_req = self.build_request(request);
        let url = format!("{}/chat/completions", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await?;

        let status = response.status();
        let body = response.bytes().await?;

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&body);
            return Err(anyhow::anyhow!("API error ({}): {}", status, error_text));
        }

        let chat_resp: openai::ChatCompletionResponse = serde_json::from_slice(&body)
            .map_err(|e| anyhow::anyhow!("failed to parse response: {} - body: {}", e, String::from_utf8_lossy(&body)))?;

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