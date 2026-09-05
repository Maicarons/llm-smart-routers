use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 内部统一请求模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    pub model: String,
    pub messages: Vec<UnifiedMessage>,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub tools: Vec<UnifiedTool>,
    pub stream: bool,
    pub stop_sequences: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub role: UnifiedRole,
    pub content: UnifiedContent,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnifiedRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnifiedContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text { text: String },
    Image { url: String, detail: Option<String> },
    ToolResult { id: String, content: String },
    ToolCall { id: String, name: String, arguments: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTool {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

/// 内部统一响应模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
    pub tool_calls: Vec<ToolCallInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// 流式事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    Text { delta: String },
    ToolCall { id: String, name: String, arguments: String },
    Finish { reason: FinishReason, usage: Option<TokenUsage> },
    Error { message: String },
}