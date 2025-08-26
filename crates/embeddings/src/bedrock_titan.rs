use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::Client as BedrockClient;
use crate::bedrock_common::invoke_bedrock;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

#[derive(Debug, Clone)]
pub struct BedrockTitanConfig {
    pub model_id: String,
    pub aws_region: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub output_embedding_length: Option<u32>,
}

impl Default for BedrockTitanConfig {
    fn default() -> Self {
        Self {
            model_id: "amazon.titan-embed-text-v2:0".to_string(),
            aws_region: "us-east-1".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            output_embedding_length: Some(1024),
        }
    }
}

#[derive(Debug, Serialize)]
struct TitanEmbedInput {
    #[serde(rename = "inputText")]
    input_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TitanEmbedOutput {
    embedding: Vec<f32>,
}

pub struct BedrockTitanClient {
    config: BedrockTitanConfig,
    client: BedrockClient,
}

impl BedrockTitanClient {
    pub async fn new(config: BedrockTitanConfig) -> Result<Self> {
        // Validate model id hint
        if !config.model_id.starts_with("amazon.titan-embed-text-") {
            warn!(
                "Model ID '{}' may not be a Titan embedding model",
                config.model_id
            );
        }

        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(config.aws_region.clone()))
            .load()
            .await;

        let client = BedrockClient::new(&aws_config);
        Ok(Self { config, client })
    }

    pub async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let embedding = self.embed_one(&text).await?;
            results.push(embedding);
        }
        Ok(results)
    }

    async fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let request = TitanEmbedInput {
            input_text: text.to_string(),
            dimensions: self.config.output_embedding_length,
        };
        let body = serde_json::to_vec(&request)?;
        let bytes = invoke_bedrock(&self.client, &self.config.model_id, body).await?;
        let parsed: TitanEmbedOutput = serde_json::from_slice(&bytes).map_err(|e| {
            if let Ok(s) = std::str::from_utf8(&bytes) {
                error!("Failed to parse Titan response JSON: {} | Raw: {}", e, s);
            }
            e
        })?;
        
        Ok(parsed.embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_titan_request_schema() {
        let req = TitanEmbedInput {
            input_text: "hello".to_string(),
            dimensions: Some(1024),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("inputText"));
        assert!(json.contains("1024"));
    }
}
