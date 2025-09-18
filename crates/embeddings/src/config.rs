use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: Option<String>,
    pub aws_region: Option<String>,
    pub dimensions: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_embedding_config() {
        let config = EmbeddingConfig {
            provider: "bedrock-cohere".to_string(),
            model: Some("cohere.embed-multilingual-v3".to_string()),
            aws_region: Some("us-east-1".to_string()),
            dimensions: Some(1024),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EmbeddingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn should_serialize_embedding_config_with_minimal_fields() {
        let config = EmbeddingConfig {
            provider: "bedrock-cohere".to_string(),
            model: None,
            aws_region: None,
            dimensions: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EmbeddingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
