use crate::anthropic::*;
use crate::openai::*;
use crate::responses::*;
use crate::unified::*;

/// 协议转换器 trait
pub trait Converter: Send + Sync {
    /// 将外部请求转换为内部统一格式
    fn to_unified_request(&self, body: &[u8]) -> anyhow::Result<UnifiedRequest>;
    /// 将内部统一响应转换为外部格式
    fn from_unified_response(&self, resp: &UnifiedResponse) -> anyhow::Result<Vec<u8>>;
    /// 将流式事件转换为外部格式
    fn to_stream_bytes(&self, event: &StreamEvent) -> anyhow::Result<Vec<u8>>;
}

/// OpenAI Chat Completions 转换器
pub struct OpenAI;

impl Converter for OpenAI {
    fn to_unified_request(&self, body: &[u8]) -> anyhow::Result<UnifiedRequest> {
        let req: ChatCompletionRequest = serde_json::from_slice(body)?;
        let mut messages = Vec::new();
        let mut system_prompt = None;

        for msg in &req.messages {
            let role = match msg.role.as_str() {
                "system" => {
                    if system_prompt.is_none() {
                        system_prompt = msg.content.as_str().map(|s| s.to_string());
                    }
                    UnifiedRole::System
                }
                "user" => UnifiedRole::User,
                "assistant" => UnifiedRole::Assistant,
                "tool" => UnifiedRole::Tool,
                _ => UnifiedRole::User,
            };

            messages.push(UnifiedMessage {
                role,
                content: UnifiedContent::Text(msg.content.to_string()),
                name: msg.name.clone(),
            });
        }

        Ok(UnifiedRequest {
            model: req.model,
            messages,
            system_prompt,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            top_p: req.top_p,
            tools: vec![],
            stream: req.stream.unwrap_or(false),
            stop_sequences: req.stop.unwrap_or_default(),
            metadata: Default::default(),
        })
    }

    fn from_unified_response(&self, resp: &UnifiedResponse) -> anyhow::Result<Vec<u8>> {
        let response = ChatCompletionResponse {
            id: resp.id.clone(),
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            model: resp.model.clone(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatResponseMessage {
                    role: "assistant".to_string(),
                    content: Some(resp.content.clone()),
                    tool_calls: if resp.tool_calls.is_empty() {
                        None
                    } else {
                        Some(
                            resp.tool_calls
                                .iter()
                                .map(|tc| ChatToolCall {
                                    id: tc.id.clone(),
                                    type_field: "function".to_string(),
                                    function: ChatFunction {
                                        name: tc.name.clone(),
                                        arguments: tc.arguments.clone(),
                                    },
                                })
                                .collect(),
                        )
                    },
                },
                finish_reason: Some(match resp.finish_reason {
                    FinishReason::Stop => "stop".to_string(),
                    FinishReason::Length => "length".to_string(),
                    FinishReason::ToolCalls => "tool_calls".to_string(),
                    FinishReason::ContentFilter => "content_filter".to_string(),
                    FinishReason::Error => "error".to_string(),
                }),
            }],
            usage: ChatUsage {
                prompt_tokens: resp.usage.prompt_tokens,
                completion_tokens: resp.usage.completion_tokens,
                total_tokens: resp.usage.total_tokens,
            },
        };
        Ok(serde_json::to_vec(&response)?)
    }

    fn to_stream_bytes(&self, event: &StreamEvent) -> anyhow::Result<Vec<u8>> {
        match event {
            StreamEvent::Text { delta } => {
                let chunk = ChatCompletionChunk {
                    id: "chatcmpl-".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    model: "".to_string(),
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: Some("assistant".to_string()),
                            content: Some(delta.clone()),
                            tool_calls: None,
                        },
                        finish_reason: None,
                    }],
                };
                let data = serde_json::to_vec(&chunk)?;
                Ok(format!("data: {}\n\n", String::from_utf8_lossy(&data)).into_bytes())
            }
            StreamEvent::Finish { reason, usage: _ } => {
                let chunk = ChatCompletionChunk {
                    id: "chatcmpl-".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    model: "".to_string(),
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: None,
                            tool_calls: None,
                        },
                        finish_reason: Some(match reason {
                            FinishReason::Stop => "stop".to_string(),
                            FinishReason::Length => "length".to_string(),
                            FinishReason::ToolCalls => "tool_calls".to_string(),
                            FinishReason::ContentFilter => "content_filter".to_string(),
                            FinishReason::Error => "error".to_string(),
                        }),
                    }],
                };
                let data = serde_json::to_vec(&chunk)?;
                Ok(format!(
                    "data: {}\n\ndata: [DONE]\n\n",
                    String::from_utf8_lossy(&data)
                )
                .into_bytes())
            }
            _ => Ok(format!("data: [DONE]\n\n").into_bytes()),
        }
    }
}

/// Anthropic Messages 转换器
pub struct AnthropicMessages;

impl Converter for AnthropicMessages {
    fn to_unified_request(&self, body: &[u8]) -> anyhow::Result<UnifiedRequest> {
        let req: MessagesRequest = serde_json::from_slice(body)?;
        let mut messages = Vec::new();

        for msg in &req.messages {
            let role = match msg.role.as_str() {
                "user" => UnifiedRole::User,
                "assistant" => UnifiedRole::Assistant,
                _ => UnifiedRole::User,
            };
            let content = match &msg.content {
                AnthropicContent::Text(t) => UnifiedContent::Text(t.clone()),
                AnthropicContent::Blocks(_) => UnifiedContent::Text("".to_string()),
            };
            messages.push(UnifiedMessage {
                role,
                content,
                name: None,
            });
        }

        Ok(UnifiedRequest {
            model: req.model,
            messages,
            system_prompt: req.system.clone(),
            max_tokens: Some(req.max_tokens),
            temperature: req.temperature,
            top_p: req.top_p,
            tools: vec![],
            stream: req.stream.unwrap_or(false),
            stop_sequences: req.stop_sequences.unwrap_or_default(),
            metadata: Default::default(),
        })
    }

    fn from_unified_response(&self, resp: &UnifiedResponse) -> anyhow::Result<Vec<u8>> {
        let response = MessagesResponse {
            id: resp.id.clone(),
            type_field: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![ContentBlock::Text {
                text: resp.content.clone(),
            }],
            model: resp.model.clone(),
            stop_reason: Some(match resp.finish_reason {
                FinishReason::Stop => "end_turn".to_string(),
                FinishReason::Length => "max_tokens".to_string(),
                FinishReason::ToolCalls => "tool_use".to_string(),
                FinishReason::ContentFilter => "end_turn".to_string(),
                FinishReason::Error => "end_turn".to_string(),
            }),
            stop_sequence: None,
            usage: AnthropicUsage {
                input_tokens: resp.usage.prompt_tokens,
                output_tokens: resp.usage.completion_tokens,
            },
        };
        Ok(serde_json::to_vec(&response)?)
    }

    fn to_stream_bytes(&self, event: &StreamEvent) -> anyhow::Result<Vec<u8>> {
        match event {
            StreamEvent::Text { delta } => {
                let event = AnthropicStreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: ContentBlockDelta::TextDelta {
                        text: delta.clone(),
                    },
                };
                let data = serde_json::to_vec(&event)?;
                Ok(format!("data: {}\n\n", String::from_utf8_lossy(&data)).into_bytes())
            }
            StreamEvent::Finish { reason, usage } => {
                let stop_reason = match reason {
                    FinishReason::Stop => "end_turn".to_string(),
                    FinishReason::Length => "max_tokens".to_string(),
                    FinishReason::ToolCalls => "tool_use".to_string(),
                    _ => "end_turn".to_string(),
                };
                let event = AnthropicStreamEvent::MessageDelta {
                    delta: MessageDelta {
                        stop_reason: Some(stop_reason),
                        stop_sequence: None,
                    },
                    usage: usage.clone().map(|u| AnthropicUsage {
                        input_tokens: u.prompt_tokens,
                        output_tokens: u.completion_tokens,
                    }),
                };
                let data = serde_json::to_vec(&event)?;
                Ok(format!("data: {}\n\n", String::from_utf8_lossy(&data)).into_bytes())
            }
            _ => Ok(format!("data: [DONE]\n\n").into_bytes()),
        }
    }
}

/// OpenAI Responses 转换器
pub struct OpenAIResponses;

impl Converter for OpenAIResponses {
    fn to_unified_request(&self, body: &[u8]) -> anyhow::Result<UnifiedRequest> {
        let req: ResponsesRequest = serde_json::from_slice(body)?;
        let input_text = match &req.input {
            ResponseInput::Text(t) => t.clone(),
            ResponseInput::Items(items) => items
                .iter()
                .filter_map(|item| match item {
                    InputItem::User { content } => Some(content.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };

        let messages = vec![UnifiedMessage {
            role: UnifiedRole::User,
            content: UnifiedContent::Text(input_text),
            name: None,
        }];

        Ok(UnifiedRequest {
            model: req.model,
            messages,
            system_prompt: req.instructions,
            max_tokens: req.max_output_tokens,
            temperature: req.temperature,
            top_p: req.top_p,
            tools: vec![],
            stream: req.stream.unwrap_or(false),
            stop_sequences: vec![],
            metadata: Default::default(),
        })
    }

    fn from_unified_response(&self, resp: &UnifiedResponse) -> anyhow::Result<Vec<u8>> {
        let response = ResponsesResponse {
            id: resp.id.clone(),
            object: "response".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            model: resp.model.clone(),
            status: match resp.finish_reason {
                FinishReason::Stop => "completed".to_string(),
                FinishReason::Length => "incomplete".to_string(),
                FinishReason::ToolCalls => "incomplete".to_string(),
                FinishReason::ContentFilter => "incomplete".to_string(),
                FinishReason::Error => "failed".to_string(),
            },
            output: vec![ResponseOutputItem::Text {
                id: "resp-".to_string(),
                content: vec![ResponseTextContent {
                    type_field: "text".to_string(),
                    text: resp.content.clone(),
                }],
            }],
            usage: ResponseUsage {
                input_tokens: resp.usage.prompt_tokens,
                output_tokens: resp.usage.completion_tokens,
                total_tokens: resp.usage.total_tokens,
            },
            error: None,
        };
        Ok(serde_json::to_vec(&response)?)
    }

    fn to_stream_bytes(&self, event: &StreamEvent) -> anyhow::Result<Vec<u8>> {
        match event {
            StreamEvent::Text { delta } => Ok(format!("data: {}\n\n", delta).into_bytes()),
            StreamEvent::Finish { .. } => Ok(format!("data: [DONE]\n\n").into_bytes()),
            _ => Ok(format!("data: [DONE]\n\n").into_bytes()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_to_unified() {
        let body = r#"{
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": "You are helpful"},
                {"role": "user", "content": "Hello"}
            ],
            "max_tokens": 100,
            "stream": true
        }"#;
        let converter = OpenAI;
        let req = converter.to_unified_request(body.as_bytes()).unwrap();
        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.system_prompt, Some("You are helpful".to_string()));
        assert_eq!(req.messages.len(), 2);
        assert!(req.stream);
    }

    #[test]
    fn test_anthropic_to_unified() {
        let body = r#"{
            "model": "claude-3-5-sonnet",
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 100,
            "system": "You are Claude"
        }"#;
        let converter = AnthropicMessages;
        let req = converter.to_unified_request(body.as_bytes()).unwrap();
        assert_eq!(req.model, "claude-3-5-sonnet");
        assert_eq!(req.system_prompt, Some("You are Claude".to_string()));
    }

    #[test]
    fn test_openai_roundtrip() {
        let converter = OpenAI;
        let body = r#"{
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "Hi"}],
            "max_tokens": 50
        }"#;
        let req = converter.to_unified_request(body.as_bytes()).unwrap();
        let resp = UnifiedResponse {
            id: "test-1".to_string(),
            model: "gpt-4o".to_string(),
            content: "Hello!".to_string(),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            tool_calls: vec![],
        };
        let output = converter.from_unified_response(&resp).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(parsed["choices"][0]["message"]["content"], "Hello!");
        assert_eq!(parsed["choices"][0]["finish_reason"], "stop");
    }

    #[test]
    fn test_anthropic_roundtrip() {
        let converter = AnthropicMessages;
        let body = r#"{
            "model": "claude-3-5-sonnet",
            "messages": [{"role": "user", "content": "Hi"}],
            "max_tokens": 100
        }"#;
        let req = converter.to_unified_request(body.as_bytes()).unwrap();
        let resp = UnifiedResponse {
            id: "test-2".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            content: "Hello!".to_string(),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            tool_calls: vec![],
        };
        let output = converter.from_unified_response(&resp).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(parsed["content"][0]["text"], "Hello!");
        assert_eq!(parsed["stop_reason"], "end_turn");
    }
}
