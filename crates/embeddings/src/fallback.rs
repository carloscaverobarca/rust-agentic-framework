use anyhow::Result;

/// Fallback embeddings provider that returns deterministic zero vectors
/// Used for offline testing and development when no real embedding API is available
pub struct FallbackEmbeddingProvider {
    embedding_dim: usize,
}

impl FallbackEmbeddingProvider {
    pub fn new(embedding_dim: usize) -> Self {
        Self { embedding_dim }
    }

    /// Creates embeddings provider with standard embedding dimension (1024)
    pub fn with_standard_dimension() -> Self {
        Self::new(1024)
    }

    /// Generate fallback embeddings as deterministic but slightly varying vectors for the given texts
    /// Each text gets a different vector to avoid all identical embeddings
    pub async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let mut embeddings = Vec::new();
        for (i, text) in texts.iter().enumerate() {
            // Create slightly different embeddings based on text hash and index
            let hash = text.len() as u32 + i as u32;
            let base_value = (hash % 100) as f32 / 1000.0; // 0.000 to 0.099

            let mut embedding = vec![base_value; self.embedding_dim];
            // Add slight variation in first few dimensions
            if !embedding.is_empty() {
                embedding[0] += 0.01;
            }
            if embedding.len() > 1 {
                embedding[1] += 0.02;
            }
            if embedding.len() > 2 {
                embedding[2] += (i as f32) * 0.001;
            }

            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    /// Get the embedding dimension used by this provider
    pub fn embedding_dimension(&self) -> usize {
        self.embedding_dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_fallback_provider_with_custom_dimension() {
        let provider = FallbackEmbeddingProvider::new(512);
        assert_eq!(provider.embedding_dimension(), 512);
    }

    #[tokio::test]
    async fn should_create_fallback_provider_with_standard_dimension() {
        let provider = FallbackEmbeddingProvider::with_standard_dimension();
        assert_eq!(provider.embedding_dimension(), 1024);
    }

    #[tokio::test]
    async fn should_return_empty_embeddings_for_empty_input() {
        let provider = FallbackEmbeddingProvider::new(768);
        let result = provider.embed(vec![]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn should_return_deterministic_embeddings_for_single_text() {
        let provider = FallbackEmbeddingProvider::new(3);
        let result = provider.embed(vec!["test text".to_string()]).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
        // Embedding should be deterministic but not all zeros
        assert!(result[0].iter().any(|&x| x != 0.0));
    }

    #[tokio::test]
    async fn should_return_different_embeddings_for_multiple_texts() {
        let provider = FallbackEmbeddingProvider::new(3);
        let texts = vec![
            "first text".to_string(),
            "second text".to_string(),
            "third text".to_string(),
        ];

        let result = provider.embed(texts).await.unwrap();

        assert_eq!(result.len(), 3);
        // Each embedding should be different
        assert_ne!(result[0], result[1]);
        assert_ne!(result[1], result[2]);
        assert_ne!(result[0], result[2]);
    }

    #[tokio::test]
    async fn should_return_correct_dimension_for_standard_fallback() {
        let provider = FallbackEmbeddingProvider::with_standard_dimension();
        let result = provider.embed(vec!["test".to_string()]).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1024);
        // Should have some non-zero values
        assert!(result[0].iter().any(|&x| x != 0.0));
    }

    #[tokio::test]
    async fn should_be_deterministic() {
        let provider = FallbackEmbeddingProvider::new(5);
        let texts = vec!["same text".to_string()];

        let result1 = provider.embed(texts.clone()).await.unwrap();
        let result2 = provider.embed(texts).await.unwrap();

        assert_eq!(result1, result2);
        // Should be deterministic but not all zeros
        assert!(result1[0].iter().any(|&x| x != 0.0));
    }
}
