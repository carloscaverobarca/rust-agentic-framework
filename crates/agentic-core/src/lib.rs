use serde::{Deserialize, Serialize};

pub mod session;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: Option<String>,
    pub aws_region: Option<String>,
    pub dimensions: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_role_enum() {
        let user_role = Role::User;
        let json = serde_json::to_string(&user_role).unwrap();
        assert_eq!(json, "\"User\"");

        let assistant_role = Role::Assistant;
        let json = serde_json::to_string(&assistant_role).unwrap();
        assert_eq!(json, "\"Assistant\"");

        let tool_role = Role::Tool;
        let json = serde_json::to_string(&tool_role).unwrap();
        assert_eq!(json, "\"Tool\"");
    }

    #[test]
    fn should_deserialize_role_enum() {
        let role: Role = serde_json::from_str("\"User\"").unwrap();
        assert_eq!(role, Role::User);

        let role: Role = serde_json::from_str("\"Assistant\"").unwrap();
        assert_eq!(role, Role::Assistant);

        let role: Role = serde_json::from_str("\"Tool\"").unwrap();
        assert_eq!(role, Role::Tool);
    }

    #[test]
    fn should_serialize_message_struct() {
        let message = Message {
            role: Role::User,
            content: "Hello, world!".to_string(),
            name: None,
        };

        let json = serde_json::to_string(&message).unwrap();
        let expected = r#"{"role":"User","content":"Hello, world!","name":null}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn should_serialize_message_with_name() {
        let message = Message {
            role: Role::Tool,
            content: "Tool result".to_string(),
            name: Some("file_summarizer".to_string()),
        };

        let json = serde_json::to_string(&message).unwrap();
        let expected = r#"{"role":"Tool","content":"Tool result","name":"file_summarizer"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn should_deserialize_message_struct() {
        let json = r#"{"role":"Assistant","content":"Hello back!","name":null}"#;
        let message: Message = serde_json::from_str(json).unwrap();

        assert_eq!(message.role, Role::Assistant);
        assert_eq!(message.content, "Hello back!");
        assert_eq!(message.name, None);
    }
}
