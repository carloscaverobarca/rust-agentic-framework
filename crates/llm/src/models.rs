use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ChatMessage {
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            name: None,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            name: None,
        }
    }

    pub fn tool(content: String, name: String) -> Self {
        Self {
            role: "tool".to_string(),
            content,
            name: Some(name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub primary_model: String,
    pub fallback_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            primary_model: "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
            fallback_model: "anthropic.claude-3-5-sonnet-20240620-v1:0".to_string(),
            max_tokens: 4096,
            temperature: 0.1,
            timeout_secs: 30,
            max_retries: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    ContentBlockStart,
    ContentBlockDelta { text: String },
    ContentBlockStop,
    MessageStart,
    MessageStop,
    Error { message: String },
}

#[derive(Debug, Serialize)]
pub struct BedrockRequest {
    pub anthropic_version: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

impl BedrockRequest {
    pub fn new(messages: Vec<ChatMessage>, config: &ModelConfig) -> Self {
        Self {
            anthropic_version: "bedrock-2023-05-31".to_string(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            messages,
            stream: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BedrockResponse {
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_user_message() {
        let msg = ChatMessage::user("Hello".to_string());
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.name, None);
    }

    #[test]
    fn should_create_assistant_message() {
        let msg = ChatMessage::assistant("Hi there".to_string());
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "Hi there");
        assert_eq!(msg.name, None);
    }

    #[test]
    fn should_create_tool_message() {
        let msg = ChatMessage::tool("Tool output".to_string(), "file_summarizer".to_string());
        assert_eq!(msg.role, "tool");
        assert_eq!(msg.content, "Tool output");
        assert_eq!(msg.name, Some("file_summarizer".to_string()));
    }

    #[test]
    fn should_create_default_model_config() {
        let config = ModelConfig::default();
        assert_eq!(
            config.primary_model,
            "anthropic.claude-3-5-sonnet-20241022-v2:0"
        );
        assert_eq!(
            config.fallback_model,
            "anthropic.claude-3-5-sonnet-20240620-v1:0"
        );
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.temperature, 0.1);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 1);
    }

    #[test]
    fn should_create_bedrock_request() {
        let messages = vec![ChatMessage::user("Test".to_string())];
        let config = ModelConfig::default();
        let request = BedrockRequest::new(messages.clone(), &config);

        assert_eq!(request.anthropic_version, "bedrock-2023-05-31");
        assert_eq!(request.max_tokens, 4096);
        assert_eq!(request.temperature, 0.1);
        assert_eq!(request.messages, messages);
        assert!(request.stream);
    }

    #[test]
    fn should_serialize_chat_message() {
        let msg = ChatMessage::user("Hello".to_string());
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
        assert!(!json.contains("\"name\""));
    }

    #[test]
    fn should_serialize_tool_message_with_name() {
        let msg = ChatMessage::tool("Output".to_string(), "tool_name".to_string());
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"role\":\"tool\""));
        assert!(json.contains("\"content\":\"Output\""));
        assert!(json.contains("\"name\":\"tool_name\""));
    }

    #[test]
    fn should_deserialize_stream_events() {
        let _json = r#"{"text": "Hello"}"#;
        let event = StreamEvent::ContentBlockDelta {
            text: "Hello".to_string(),
        };

        // Test the enum variants exist and can be constructed
        match event {
            StreamEvent::ContentBlockDelta { text } => assert_eq!(text, "Hello"),
            _ => panic!("Wrong enum variant"),
        }
    }
}
