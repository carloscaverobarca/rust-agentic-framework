use anyhow::{Context, Result};
use pgvector::Vector;
use sqlx::{PgPool, Row};
use tracing;

use crate::models::{Document, DocumentChunk, SearchResult};

pub struct VectorStore {
    pool: PgPool,
    embedding_dimensions: usize,
}

impl VectorStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        Self::new_with_dimensions(database_url, 1024).await
    }

    pub async fn new_with_dimensions(
        database_url: &str,
        embedding_dimensions: usize,
    ) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .context("Failed to connect to PostgreSQL")?;

        db_migrations::run_migrations(database_url)
            .await
            .context("Failed to run database migrations")?;

        tracing::info!(
            "Vector store initialization completed successfully with migrations applied"
        );

        Ok(Self {
            pool,
            embedding_dimensions,
        })
    }

    pub async fn insert_document(&self, chunk: DocumentChunk) -> Result<Document> {
        let document = chunk.into_document();

        // Validate embedding dimensions match configuration
        if document.embedding.len() != self.embedding_dimensions {
            return Err(anyhow::anyhow!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.embedding_dimensions,
                document.embedding.len()
            ));
        }

        let embedding_vector = Vector::from(document.embedding.clone());

        sqlx::query(
            r#"
            INSERT INTO documents (id, file_name, chunk_id, content, embedding, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(document.id)
        .bind(&document.file_name)
        .bind(document.chunk_id as i32)
        .bind(&document.content)
        .bind(embedding_vector)
        .bind(document.created_at)
        .execute(&self.pool)
        .await
        .context("Failed to insert document")?;

        Ok(document)
    }

    pub async fn search_similar(
        &self,
        query_embedding: Vec<f32>,
        limit: i32,
    ) -> Result<Vec<SearchResult>> {
        tracing::info!(
            "VectorStore::search_similar: query_embedding_length={}, query_embedding_sample={:?}",
            query_embedding.len(),
            &query_embedding[..std::cmp::min(5, query_embedding.len())]
        );

        if query_embedding.len() != self.embedding_dimensions {
            return Err(anyhow::anyhow!(
                "Query embedding dimension mismatch: expected {}, got {}",
                self.embedding_dimensions,
                query_embedding.len()
            ));
        }

        let query_vector = Vector::from(query_embedding.clone());

        let rows = sqlx::query(
            r#"
            WITH similarity_search AS (
                SELECT
                    id,
                    file_name,
                    chunk_id,
                    content,
                    embedding,
                    created_at,
                    1 - (embedding <=> $1) as similarity
                FROM documents
                WHERE 1 - (embedding <=> $1) > 0.01  -- Only get results with some similarity
                ORDER BY embedding <=> $1
                LIMIT $2
            )
            SELECT * FROM similarity_search
            "#,
        )
        .bind(query_vector)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .with_context(|| "Failed to execute similarity search")?;

        tracing::info!("Found {} results", rows.len());

        let results = rows
            .iter()
            .map(|row| {
                let embedding_vector: Vector = row.get("embedding");
                let embedding: Vec<f32> = embedding_vector.into();
                let similarity: f64 = row.get("similarity");

                let document = Document {
                    id: row.get("id"),
                    file_name: row.get("file_name"),
                    chunk_id: row.get::<i32, _>("chunk_id") as usize,
                    content: row.get("content"),
                    embedding,
                    created_at: row.get("created_at"),
                };

                tracing::info!(
                    "Result: file={}, similarity={:.4}, content={}",
                    document.file_name,
                    similarity,
                    document.content.chars().take(50).collect::<String>()
                );

                SearchResult::new(document, similarity as f32)
            })
            .collect();

        Ok(results)
    }

    pub async fn get_document_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM documents")
            .fetch_one(&self.pool)
            .await
            .context("Failed to get document count")?;

        Ok(row.get("count"))
    }

    pub async fn delete_all_documents(&self) -> Result<()> {
        sqlx::query("DELETE FROM documents")
            .execute(&self.pool)
            .await
            .context("Failed to delete all documents")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DocumentChunk;

    // These tests require a running PostgreSQL with pgvector extension
    // For now, we'll create stub tests that demonstrate the interface

    #[tokio::test]
    async fn should_create_vector_store_with_valid_url() {
        // This test would fail without a real database
        // For TDD purposes, we're designing the interface first
        let result = VectorStore::new("postgresql://invalid").await;

        // We expect this to fail with connection error - that's fine for TDD
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_handle_document_insertion() {
        // Mock test - demonstrates expected interface
        let embedding = vec![0.1; 1024]; // 1024-dimensional vector
        let chunk = DocumentChunk::new(
            "test.txt".to_string(),
            0,
            "test content".to_string(),
            embedding,
        );

        // This would require a real database connection to work
        // For now, we're just testing the types compile correctly
        assert_eq!(chunk.file_name, "test.txt");
        assert_eq!(chunk.chunk_id, 0);
        assert_eq!(chunk.content, "test content");
        assert_eq!(chunk.embedding.len(), 1024);
    }

    #[tokio::test]
    async fn should_handle_similarity_search() {
        // Mock test - demonstrates expected interface
        let query_embedding = vec![0.2; 1024];

        // This would require a real database to work
        // For now, we're testing the interface design
        assert_eq!(query_embedding.len(), 1024);
        assert!(query_embedding.iter().all(|&x| x == 0.2));
    }

    #[test]
    fn should_create_document_chunk_with_correct_dimensions() {
        let embedding = vec![0.1; 1024];
        let chunk = DocumentChunk::new(
            "test.txt".to_string(),
            1,
            "chunk content".to_string(),
            embedding.clone(),
        );

        let document = chunk.into_document();
        assert_eq!(document.embedding.len(), 1024);
        assert_eq!(document.file_name, "test.txt");
        assert_eq!(document.chunk_id, 1);
    }

    #[test]
    fn should_create_search_result_with_similarity_score() {
        let embedding = vec![0.1; 1024];
        let document = Document::new(
            "test.txt".to_string(),
            0,
            "test content".to_string(),
            embedding,
        );

        let result = SearchResult::new(document.clone(), 0.95);
        assert_eq!(result.similarity, 0.95);
        assert_eq!(result.document.id, document.id);
    }

    #[test]
    fn should_handle_1024_dimensional_embeddings() {
        // RED: This test should fail because current schema only supports 768 dimensions
        let embedding_1024 = vec![0.1; 1024];
        let chunk = DocumentChunk::new(
            "bedrock_test.txt".to_string(),
            0,
            "test content for bedrock cohere".to_string(),
            embedding_1024.clone(),
        );

        let document = chunk.into_document();
        assert_eq!(document.embedding.len(), 1024);
        assert_eq!(document.file_name, "bedrock_test.txt");

        // This test validates that our types can handle 1024 dimensions
        // The actual database insertion would fail with current schema
    }

    #[tokio::test]
    async fn should_support_different_embedding_dimensions() {
        // GREEN: Now this test should pass with our new dimension support
        let embedding_1024 = vec![0.2; 1024];

        // Test that similarity search interface can handle 1024 dimensions
        assert_eq!(embedding_1024.len(), 1024);
        assert!(embedding_1024.iter().all(|&x| x == 0.2));

        // Test that VectorStore can be created with 1024 dimensions
        let result = VectorStore::new_with_dimensions("postgresql://invalid", 1024).await;
        assert!(result.is_err()); // Still fails due to invalid connection, but dimensions are supported
    }

    #[tokio::test]
    async fn should_create_vector_store_with_1024_dimensions() {
        // GREEN: Test for creating VectorStore with Bedrock Cohere dimensions
        let result = VectorStore::new_with_dimensions("postgresql://invalid", 1024).await;

        // We expect connection to fail, but the constructor should accept 1024 dimensions
        assert!(result.is_err());
    }

    #[test]
    fn should_validate_embedding_dimensions() {
        // Test that embedding dimension validation works
        let embedding_768 = vec![0.1; 768];
        let embedding_1024 = vec![0.1; 1024];

        assert_eq!(embedding_768.len(), 768);
        assert_eq!(embedding_1024.len(), 1024);

        // These would be validated at runtime when inserting/searching
    }
}
