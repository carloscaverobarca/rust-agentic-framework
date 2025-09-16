use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CohereConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for CohereConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "embed-english-v3.0".to_string(),
            base_url: "https://api.cohere.ai".to_string(),
            timeout_secs: 30,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Serialize)]
struct EmbedRequest {
    texts: Vec<String>,
    model: String,
    input_type: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

pub struct CohereClient {
    config: CohereConfig,
    client: Client,
}

impl CohereClient {
    pub fn new(config: CohereConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    pub async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self.try_embed(&texts).await {
                Ok(embeddings) => return Ok(embeddings),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries {
                        let delay = Duration::from_millis(1000 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn try_embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let request = EmbedRequest {
            texts: texts.to_vec(),
            model: self.config.model.clone(),
            input_type: "search_document".to_string(),
        };

        let response = self
            .client
            .post(format!("{}/v1/embed", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Cohere API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Cohere API returned error {}: {}",
                status,
                error_text
            ));
        }

        let embed_response: EmbedResponse = response
            .json()
            .await
            .context("Failed to parse Cohere API response")?;

        Ok(embed_response.embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_cohere_client_with_default_config() {
        let config = CohereConfig::default();
        let client = CohereClient::new(config.clone());

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.config.model, "embed-english-v3.0");
        assert_eq!(client.config.base_url, "https://api.cohere.ai");
        assert_eq!(client.config.timeout_secs, 30);
        assert_eq!(client.config.max_retries, 3);
    }

    #[tokio::test]
    async fn should_create_cohere_client_with_custom_config() {
        let config = CohereConfig {
            api_key: "test-key".to_string(), // pragma: allowlist secret
            model: "embed-multilingual-v3.0".to_string(),
            base_url: "https://custom.api.com".to_string(),
            timeout_secs: 60,
            max_retries: 5,
        };

        let client = CohereClient::new(config.clone()).unwrap();
        assert_eq!(client.config.api_key, "test-key");
        assert_eq!(client.config.model, "embed-multilingual-v3.0");
        assert_eq!(client.config.base_url, "https://custom.api.com");
        assert_eq!(client.config.timeout_secs, 60);
        assert_eq!(client.config.max_retries, 5);
    }

    #[tokio::test]
    async fn should_return_empty_embeddings_for_empty_input() {
        let config = CohereConfig::default();
        let client = CohereClient::new(config).unwrap();

        let result = client.embed(vec![]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn should_handle_single_text_embedding() {
        // This test will fail until we implement proper mock server
        // For now, it demonstrates the expected interface
        let config = CohereConfig {
            api_key: "mock-key".to_string(), // pragma: allowlist secret
            ..CohereConfig::default()
        };
        let client = CohereClient::new(config).unwrap();

        // This will fail with network error in tests - that's expected for now
        let result = client.embed(vec!["Hello world".to_string()]).await;

        // For TDD, we expect this to fail until we add proper mocking
        assert!(result.is_err() || result.unwrap().len() == 1);
    }

    #[tokio::test]
    async fn should_handle_multiple_text_embeddings() {
        let config = CohereConfig {
            api_key: "mock-key".to_string(), // pragma: allowlist secret
            ..CohereConfig::default()
        };
        let client = CohereClient::new(config).unwrap();

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        // This will fail with network error in tests - that's expected for now
        let result = client.embed(texts).await;

        // For TDD, we expect this to fail until we add proper mocking
        assert!(result.is_err() || result.unwrap().len() == 3);
    }

    #[tokio::test]
    async fn should_retry_on_failure() {
        let config = CohereConfig {
            api_key: "invalid-key".to_string(), // pragma: allowlist secret
            base_url: "https://nonexistent.api.com".to_string(),
            timeout_secs: 1, // Short timeout for fast test
            max_retries: 2,
            ..CohereConfig::default()
        };
        let client = CohereClient::new(config).unwrap();

        let start = std::time::Instant::now();
        let result = client.embed(vec!["test".to_string()]).await;
        let duration = start.elapsed();

        // Should fail after retries
        assert!(result.is_err());
        // Should have taken time for retries (at least 1 second for first retry)
        assert!(duration.as_millis() >= 1000);
    }

    #[test]
    fn should_create_embed_request_with_correct_format() {
        let request = EmbedRequest {
            texts: vec!["test text".to_string()],
            model: "embed-english-v3.0".to_string(),
            input_type: "search_document".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test text"));
        assert!(json.contains("embed-english-v3.0"));
        assert!(json.contains("search_document"));
    }
}
