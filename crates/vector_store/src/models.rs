use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
