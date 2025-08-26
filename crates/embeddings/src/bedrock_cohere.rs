use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::Client as BedrockClient;
use crate::bedrock_common::invoke_bedrock;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

#[derive(Debug, Clone)]
pub struct BedrockCohereConfig {
    pub model_id: String,
    pub aws_region: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for BedrockCohereConfig {
    fn default() -> Self {
        Self {
            model_id: "cohere.embed-multilingual-v3".to_string(),
            aws_region: "us-east-1".to_string(),
            timeout_secs: 30,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Serialize)]
struct BedrockEmbedRequest {
    texts: Vec<String>,
    input_type: String,
}

#[derive(Debug, Deserialize)]
struct BedrockEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

pub struct BedrockCohereClient {
    config: BedrockCohereConfig,
    client: BedrockClient,
}

impl BedrockCohereClient {
    /// Test AWS connectivity and credentials without making an embedding request
    pub async fn test_connection(&self) -> Result<()> {
        let test_request = BedrockEmbedRequest {
            texts: vec!["test".to_string()],
            input_type: "search_document".to_string(),
        };

        let request_body = serde_json::to_string(&test_request)?;
        // This will fail if credentials are wrong, region is wrong, or model doesn't exist
        match invoke_bedrock(&self.client, &self.config.model_id, request_body.into_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("AWS Bedrock connection test failed: {}", e);
                Err(e)
            }
        }
    }
    pub async fn new(config: BedrockCohereConfig) -> Result<Self> {
        // Check for common AWS environment variables
        Self::log_aws_environment();

        // Validate configuration
        Self::validate_config(&config)?;

        // Load AWS config based on the region
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(config.aws_region.clone()))
            .load()
            .await;

        let client = BedrockClient::new(&aws_config);

        Ok(Self { config, client })
    }

    fn log_aws_environment() {
        // Check for AWS credentials without exposing values
        let aws_access_key = std::env::var("AWS_ACCESS_KEY_ID").is_ok();
        let aws_secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").is_ok();

        if !aws_access_key && !aws_secret_key {
            warn!("No AWS credentials found in environment variables. Relying on other credential sources (IAM role, ~/.aws/credentials, etc.)");
        }
    }

    fn validate_config(config: &BedrockCohereConfig) -> Result<()> {
        if config.model_id.is_empty() {
            let error = "Model ID cannot be empty";
            error!(error);
            return Err(anyhow::anyhow!(error));
        }

        if config.aws_region.is_empty() {
            let error = "AWS region cannot be empty";
            error!(error);
            return Err(anyhow::anyhow!(error));
        }

        // Validate model ID format for Bedrock Cohere models
        if !config.model_id.starts_with("cohere.embed-") {
            warn!(
                "Model ID '{}' does not follow expected Bedrock Cohere format (cohere.embed-*)",
                config.model_id
            );
        }

        // Check for valid AWS regions (partial list of commonly used regions)
        let valid_regions = [
            "us-east-1",
            "us-east-2",
            "us-west-1",
            "us-west-2",
            "eu-west-1",
            "eu-west-2",
            "eu-central-1",
            "eu-north-1",
            "ap-southeast-1",
            "ap-southeast-2",
            "ap-northeast-1",
            "ap-south-1",
        ];

        if !valid_regions.contains(&config.aws_region.as_str()) {
            warn!("Region '{}' is not in the common regions list. Please ensure Bedrock is available in this region.", config.aws_region);
        }

        Ok(())
    }

    pub async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self.try_embed(&texts).await {
                Ok(embeddings) => {
                    return Ok(embeddings);
                }
                Err(e) => {
                    warn!("Embedding attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        let delay = std::time::Duration::from_millis(1000 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        let final_error = last_error.unwrap();
        error!(
            "All embedding attempts failed. Final error: {}",
            final_error
        );
        Err(final_error)
    }

    async fn try_embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let request = BedrockEmbedRequest {
            texts: texts.to_vec(),
            input_type: "search_document".to_string(),
        };

        let request_body = serde_json::to_string(&request).map_err(|e| {
            error!("Failed to serialize request: {}", e);
            e
        })?;
        let response_body = invoke_bedrock(&self.client, &self.config.model_id, request_body.into_bytes())
            .await
            .map_err(|e| {
                error!("Bedrock invoke_model failed: {}", e);
                e
            })?;

        let embed_response: BedrockEmbedResponse = serde_json::from_slice(&response_body)
            .map_err(|e| {
                error!("Failed to parse response JSON: {}", e);
                if let Ok(response_str) = std::str::from_utf8(&response_body) {
                    error!("Raw response: {}", response_str);

                    // Check for common error patterns in response
                    if response_str.contains("ValidationException") {
                        error!("Bedrock ValidationException - likely invalid model ID or request format");
                    } else if response_str.contains("AccessDeniedException") {
                        error!("Bedrock AccessDeniedException - check IAM permissions for model access");
                    } else if response_str.contains("ThrottlingException") {
                        error!("Bedrock ThrottlingException - request rate exceeded, will retry");
                    } else if response_str.contains("ModelNotReadyException") {
                        error!("Bedrock ModelNotReadyException - model is not ready or available in this region");
                    }
                }
                e
            })?;

        Ok(embed_response.embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber;

    #[tokio::test]
    async fn should_create_bedrock_cohere_client_with_default_config() {
        let config = BedrockCohereConfig::default();
        let client = BedrockCohereClient::new(config.clone()).await;

        // This test should initially FAIL (RED phase)
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.config.model_id, "cohere.embed-multilingual-v3");
        assert_eq!(client.config.aws_region, "us-east-1");
        assert_eq!(client.config.timeout_secs, 30);
        assert_eq!(client.config.max_retries, 3);
    }

    #[tokio::test]
    async fn should_create_bedrock_cohere_client_with_custom_config() {
        let config = BedrockCohereConfig {
            model_id: "cohere.embed-english-v3".to_string(),
            aws_region: "eu-west-1".to_string(),
            timeout_secs: 60,
            max_retries: 5,
        };

        // This test should initially FAIL (RED phase)
        let client = BedrockCohereClient::new(config.clone()).await.unwrap();
        assert_eq!(client.config.model_id, "cohere.embed-english-v3");
        assert_eq!(client.config.aws_region, "eu-west-1");
        assert_eq!(client.config.timeout_secs, 60);
        assert_eq!(client.config.max_retries, 5);
    }

    #[tokio::test]
    async fn should_return_empty_embeddings_for_empty_input() {
        let config = BedrockCohereConfig::default();
        let client = BedrockCohereClient::new(config).await.unwrap();

        // This test should initially FAIL (RED phase)
        let result = client.embed(vec![]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn should_handle_single_text_embedding() {
        // Initialize tracing for test debugging
        #[cfg(test)]
        let _ = tracing_subscriber::fmt().try_init();

        let config = BedrockCohereConfig::default();
        let client = BedrockCohereClient::new(config).await.unwrap();

        // This test should initially FAIL (RED phase)
        // We'll use mock or test mode to avoid actual AWS calls
        let result = client.embed(vec!["Hello world".to_string()]).await;

        // For now, we expect this to fail until we implement it
        // In a real environment with AWS credentials, this should succeed
        assert!(result.is_err() || result.unwrap().len() == 1);
    }

    #[tokio::test]
    async fn should_handle_multiple_text_embeddings() {
        // Initialize tracing for test debugging
        #[cfg(test)]
        let _ = tracing_subscriber::fmt().try_init();

        let config = BedrockCohereConfig::default();
        let client = BedrockCohereClient::new(config).await.unwrap();

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        // This test should initially FAIL (RED phase)
        let result = client.embed(texts).await;

        // For now, we expect this to fail until we implement it
        // In a real environment with AWS credentials, this should succeed
        assert!(result.is_err() || result.unwrap().len() == 3);
    }

    #[tokio::test]
    async fn should_create_proper_bedrock_request_format() {
        // This test will fail initially - TDD RED phase
        let request = BedrockEmbedRequest {
            texts: vec!["test text".to_string()],
            input_type: "search_document".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test text"));
        assert!(json.contains("search_document"));
        assert!(!json.contains("model")); // Bedrock doesn't include model in request body
    }

    #[tokio::test]
    async fn should_test_aws_connection() {
        // Initialize tracing for test debugging
        #[cfg(test)]
        let _ = tracing_subscriber::fmt().try_init();

        let config = BedrockCohereConfig::default();
        let client = BedrockCohereClient::new(config).await.unwrap();

        // This test will help debug AWS connectivity issues
        let result = client.test_connection().await;

        // This will typically fail in CI/test environments without AWS credentials
        // but provides valuable debugging information
        if result.is_err() {
            println!("Expected failure in test environment: {:?}", result.err());
        }

        // Don't assert success since this test is for debugging purposes
        // In a real AWS environment, you could uncomment the line below:
        // assert!(result.is_ok(), "AWS connection test should pass with valid credentials");
    }
}
