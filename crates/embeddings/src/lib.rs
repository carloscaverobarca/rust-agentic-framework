pub mod bedrock_cohere;
pub mod bedrock_common;
pub mod bedrock_titan;
pub mod chunker;
pub mod cohere;
pub mod config;
pub mod fallback;

pub use bedrock_cohere::{BedrockCohereClient, BedrockCohereConfig};
pub use bedrock_titan::{BedrockTitanClient, BedrockTitanConfig};
pub use chunker::{ChunkConfig, TextChunk, TextChunker};
pub use cohere::{CohereClient, CohereConfig};
pub use config::EmbeddingConfig;
pub use fallback::FallbackEmbeddingProvider;

use anyhow::Result;

type EmbedFuture<'a> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Vec<f32>>>> + Send + 'a>>;

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, texts: Vec<String>) -> EmbedFuture<'_>;
    fn dimension(&self) -> usize;
}

impl EmbeddingProvider for CohereClient {
    fn embed(&self, texts: Vec<String>) -> EmbedFuture<'_> {
        Box::pin(self.embed(texts))
    }
    fn dimension(&self) -> usize {
        1024
    }
}

impl EmbeddingProvider for BedrockCohereClient {
    fn embed(&self, texts: Vec<String>) -> EmbedFuture<'_> {
        Box::pin(self.embed(texts))
    }
    fn dimension(&self) -> usize {
        1024
    }
}

impl EmbeddingProvider for BedrockTitanClient {
    fn embed(&self, texts: Vec<String>) -> EmbedFuture<'_> {
        Box::pin(self.embed(texts))
    }
    fn dimension(&self) -> usize {
        1024
    }
}

impl EmbeddingProvider for FallbackEmbeddingProvider {
    fn embed(&self, texts: Vec<String>) -> EmbedFuture<'_> {
        Box::pin(self.embed(texts))
    }
    fn dimension(&self) -> usize {
        self.embedding_dimension()
    }
}

pub async fn create_embedding_provider(
    cfg: &EmbeddingConfig,
) -> Result<Box<dyn EmbeddingProvider>> {
    match cfg.provider.as_str() {
        "cohere" => {
            let cohere_cfg = CohereConfig {
                api_key: std::env::var("COHERE_API_KEY").unwrap_or_default(),
                ..CohereConfig::default()
            };
            Ok(Box::new(CohereClient::new(cohere_cfg)?))
        }
        "bedrock-cohere" => {
            let model_id = cfg
                .model
                .clone()
                .unwrap_or_else(|| "cohere.embed-multilingual-v3".to_string());
            let aws_region = cfg
                .aws_region
                .clone()
                .unwrap_or_else(|| "us-east-1".to_string());
            let br_cfg = BedrockCohereConfig {
                model_id,
                aws_region,
                ..BedrockCohereConfig::default()
            };
            Ok(Box::new(BedrockCohereClient::new(br_cfg).await?))
        }
        "bedrock-titan" => {
            let model_id = cfg
                .model
                .clone()
                .unwrap_or_else(|| "amazon.titan-embed-text-v2:0".to_string());
            let aws_region = cfg
                .aws_region
                .clone()
                .unwrap_or_else(|| "us-east-1".to_string());
            let mut br_cfg = BedrockTitanConfig {
                model_id,
                aws_region,
                ..BedrockTitanConfig::default()
            };
            if let Some(dim) = cfg.dimensions {
                br_cfg.output_embedding_length = Some(dim as u32);
            }
            Ok(Box::new(BedrockTitanClient::new(br_cfg).await?))
        }
        _ => Ok(Box::new(FallbackEmbeddingProvider::with_cohere_dimension())),
    }
}
