use crate::unified::{FinishReason, StreamEvent, TokenUsage};

/// 将 SSE 字节流解析为 StreamEvent
pub fn parse_sse_line(line: &str) -> Option<StreamEvent> {
    if line.starts_with("data: ") {
        let data = &line[6..];
        if data == "[DONE]" {
            return Some(StreamEvent::Finish {
                reason: FinishReason::Stop,
                usage: None,
            });
        }
        // 尝试解析为 OpenAI 格式
        if let Ok(chunk) = serde_json::from_str::<crate::openai::ChatCompletionChunk>(data) {
            if let Some(choice) = chunk.choices.first() {
                if let Some(content) = &choice.delta.content {
                    if !content.is_empty() {
                        return Some(StreamEvent::Text { delta: content.clone() });
                    }
                }
                if let Some(reason) = &choice.finish_reason {
                    let finish = match reason.as_str() {
                        "stop" => FinishReason::Stop,
                        "length" => FinishReason::Length,
                        "tool_calls" => FinishReason::ToolCalls,
                        "content_filter" => FinishReason::ContentFilter,
                        _ => FinishReason::Error,
                    };
                    return Some(StreamEvent::Finish { reason: finish, usage: None });
                }
            }
        }
        // 尝试解析为 Anthropic 格式
        if let Ok(event) = serde_json::from_str::<crate::anthropic::AnthropicStreamEvent>(data) {
            return match event {
                crate::anthropic::AnthropicStreamEvent::ContentBlockDelta { delta, .. } => {
                    match delta {
                        crate::anthropic::ContentBlockDelta::TextDelta { text } => {
                            Some(StreamEvent::Text { delta: text })
                        }
                        crate::anthropic::ContentBlockDelta::InputJsonDelta { partial_json } => {
                            Some(StreamEvent::ToolCall {
                                id: "".to_string(),
                                name: "".to_string(),
                                arguments: partial_json,
                            })
                        }
                    }
                }
                crate::anthropic::AnthropicStreamEvent::MessageDelta { delta, usage } => {
                    let reason = match delta.stop_reason.as_deref() {
                        Some("end_turn") => FinishReason::Stop,
                        Some("max_tokens") => FinishReason::Length,
                        Some("tool_use") => FinishReason::ToolCalls,
                        _ => FinishReason::Stop,
                    };
                    let usage_info = usage.map(|u| TokenUsage {
                        prompt_tokens: u.input_tokens,
                        completion_tokens: u.output_tokens,
                        total_tokens: u.input_tokens + u.output_tokens,
                    });
                    Some(StreamEvent::Finish { reason, usage: usage_info })
                }
                _ => None,
            };
        }
    }
    None
}

/// 将 StreamEvent 编码为 SSE 格式
pub fn encode_sse(event: &StreamEvent) -> String {
    match event {
        StreamEvent::Text { delta } => {
            format!("data: {}\n\n", serde_json::json!({"delta": delta}))
        }
        StreamEvent::Finish { reason, usage } => {
            let mut data = serde_json::json!({
                "finish_reason": match reason {
                    FinishReason::Stop => "stop",
                    FinishReason::Length => "length",
                    FinishReason::ToolCalls => "tool_calls",
                    FinishReason::ContentFilter => "content_filter",
                    FinishReason::Error => "error",
                }
            });
            if let Some(u) = usage {
                data["usage"] = serde_json::json!({
                    "prompt_tokens": u.prompt_tokens,
                    "completion_tokens": u.completion_tokens,
                    "total_tokens": u.total_tokens,
                });
            }
            format!("data: {}\n\ndata: [DONE]\n\n", data)
        }
        StreamEvent::ToolCall { id, name, arguments } => {
            format!("data: {}\n\n", serde_json::json!({
                "tool_call": {"id": id, "name": name, "arguments": arguments}
            }))
        }
        StreamEvent::Error { message } => {
            format!("data: {}\n\n", serde_json::json!({"error": message}))
        }
    }
}