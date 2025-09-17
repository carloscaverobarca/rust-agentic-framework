use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Core types moved from agentic-core crate
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

// Session data for Redis storage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionData {
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Default for SessionData {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionData {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            messages: Vec::new(),
            created_at: now,
            last_accessed: now,
        }
    }

    pub fn with_message(message: Message) -> Self {
        let mut session = Self::new();
        session.messages.push(message);
        session
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub id: Uuid,
    pub file_name: String,
    pub chunk_id: usize,
    pub content: String,
    pub embedding: Vec<f32>,
    pub created_at: DateTime<Utc>,
}

impl Document {
    pub fn new(file_name: String, chunk_id: usize, content: String, embedding: Vec<f32>) -> Self {
        Self {
            id: Uuid::new_v4(),
            file_name,
            chunk_id,
            content,
            embedding,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub file_name: String,
    pub chunk_id: usize,
    pub content: String,
    pub embedding: Vec<f32>,
}

impl DocumentChunk {
    pub fn new(file_name: String, chunk_id: usize, content: String, embedding: Vec<f32>) -> Self {
        Self {
            file_name,
            chunk_id,
            content,
            embedding,
        }
    }

    pub fn into_document(self) -> Document {
        Document::new(self.file_name, self.chunk_id, self.content, self.embedding)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub similarity: f32,
}

impl SearchResult {
    pub fn new(document: Document, similarity: f32) -> Self {
        Self {
            document,
            similarity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for core types moved from agentic-core
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

    #[test]
    fn should_create_session_data() {
        let session = SessionData::new();
        assert!(session.messages.is_empty());
        assert!(session.created_at <= Utc::now());
        assert!(session.last_accessed <= Utc::now());
    }

    #[test]
    fn should_create_session_data_with_message() {
        let message = Message {
            role: Role::User,
            content: "Hello".to_string(),
            name: None,
        };

        let session = SessionData::with_message(message.clone());
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0], message);
    }

    // Tests for existing document types
    #[test]
    fn should_create_document_with_uuid_and_timestamp() {
        let embedding = vec![0.1, 0.2, 0.3];
        let doc = Document::new(
            "test.txt".to_string(),
            0,
            "test content".to_string(),
            embedding.clone(),
        );

        assert_eq!(doc.file_name, "test.txt");
        assert_eq!(doc.chunk_id, 0);
        assert_eq!(doc.content, "test content");
        assert_eq!(doc.embedding, embedding);
        assert!(!doc.id.is_nil());
        assert!(doc.created_at <= Utc::now());
    }

    #[test]
    fn should_create_document_chunk() {
        let embedding = vec![0.1, 0.2, 0.3];
        let chunk = DocumentChunk::new(
            "test.txt".to_string(),
            1,
            "chunk content".to_string(),
            embedding.clone(),
        );

        assert_eq!(chunk.file_name, "test.txt");
        assert_eq!(chunk.chunk_id, 1);
        assert_eq!(chunk.content, "chunk content");
        assert_eq!(chunk.embedding, embedding);
    }

    #[test]
    fn should_convert_document_chunk_to_document() {
        let embedding = vec![0.1, 0.2, 0.3];
        let chunk = DocumentChunk::new(
            "test.txt".to_string(),
            1,
            "chunk content".to_string(),
            embedding.clone(),
        );

        let doc = chunk.into_document();

        assert_eq!(doc.file_name, "test.txt");
        assert_eq!(doc.chunk_id, 1);
        assert_eq!(doc.content, "chunk content");
        assert_eq!(doc.embedding, embedding);
        assert!(!doc.id.is_nil());
    }

    #[test]
    fn should_create_search_result() {
        let embedding = vec![0.1, 0.2, 0.3];
        let doc = Document::new(
            "test.txt".to_string(),
            0,
            "test content".to_string(),
            embedding,
        );

        let result = SearchResult::new(doc.clone(), 0.95);

        assert_eq!(result.document, doc);
        assert_eq!(result.similarity, 0.95);
    }

    #[test]
    fn should_serialize_and_deserialize_document() {
        let embedding = vec![0.1, 0.2, 0.3];
        let doc = Document::new(
            "test.txt".to_string(),
            0,
            "test content".to_string(),
            embedding,
        );

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: Document = serde_json::from_str(&json).unwrap();

        assert_eq!(doc, deserialized);
    }
}
