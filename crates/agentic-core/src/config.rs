use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub embedding: EmbeddingConfig,
    pub llm: LlmConfig,
    pub pgvector: PgVectorConfig,
    pub data: DataConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: Option<String>,
    pub aws_region: Option<String>,
    pub dimensions: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmConfig {
    pub primary: String,
    pub fallback: String,
}

impl LlmConfig {
    pub fn with_env_overrides(&self) -> Self {
        let primary = env::var("LLM_PRIMARY_MODEL").unwrap_or_else(|_| self.primary.clone());
        let fallback = env::var("LLM_FALLBACK_MODEL").unwrap_or_else(|_| self.fallback.clone());
        Self { primary, fallback }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PgVectorConfig {
    pub url: String,
}

impl PgVectorConfig {
    pub fn with_env_overrides(&self) -> Self {
        let url = env::var("PGVECTOR_URL").unwrap_or_else(|_| self.url.clone());
        Self { url }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataConfig {
    pub document_dir: String,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_from_env() -> anyhow::Result<Self> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| Self::default_config_path());
        Self::load(Path::new(&config_path))
    }

    pub fn default_config_path() -> String {
        "./config.toml".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn should_deserialize_config_from_toml() {
        let toml_content = r#"
[embedding]
provider = "cohere"

[llm]
primary = "claude-sonnet-v4"
fallback = "claude-sonnet-v3.7"

[pgvector]
url = "postgres://localhost:5432/chatbot"

[data]
document_dir = "./data/faq_docs"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert_eq!(config.embedding.provider, "cohere");
        assert_eq!(config.llm.primary, "claude-sonnet-v4");
        assert_eq!(config.llm.fallback, "claude-sonnet-v3.7");
        assert_eq!(config.pgvector.url, "postgres://localhost:5432/chatbot");
        assert_eq!(config.data.document_dir, "./data/faq_docs");
    }

    #[test]
    fn should_load_config_from_file() {
        let toml_content = r#"
[embedding]
provider = "cohere"

[llm]
primary = "claude-sonnet-v4"
fallback = "claude-sonnet-v3.7"

[pgvector]
url = "postgres://localhost:5432/chatbot"

[data]
document_dir = "./data/faq_docs"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::load(temp_file.path()).unwrap();

        assert_eq!(config.embedding.provider, "cohere");
        assert_eq!(config.llm.primary, "claude-sonnet-v4");
    }

    #[test]
    fn should_load_config_with_default_path() {
        let toml_content = r#"
[embedding]
provider = "cohere"

[llm]
primary = "claude-sonnet-v4"
fallback = "claude-sonnet-v3.7"

[pgvector]
url = "postgres://localhost:5432/chatbot"

[data]
document_dir = "./data/faq_docs"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_string_lossy().to_string();

        env::set_var("CONFIG_PATH", &temp_path);

        let config = Config::load_from_env().unwrap();

        assert_eq!(config.embedding.provider, "cohere");

        env::remove_var("CONFIG_PATH");
    }

    #[test]
    fn should_use_default_config_path_when_env_not_set() {
        env::remove_var("CONFIG_PATH");

        // This test just verifies the default path is used
        let default_path = Config::default_config_path();
        assert_eq!(default_path, "./config.toml");
    }

    #[test]
    fn should_return_error_for_missing_file() {
        let result = Config::load(Path::new("/non/existent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn should_return_error_for_invalid_toml() {
        let invalid_toml = "invalid toml content [[[";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(invalid_toml.as_bytes()).unwrap();

        let result = Config::load(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn should_deserialize_bedrock_cohere_config() {
        // TDD RED phase - this test will fail initially
        let toml_content = r#"
[embedding]
provider = "bedrock-cohere"
model = "cohere.embed-multilingual-v3"
aws_region = "us-east-1"

[llm]
primary = "claude-sonnet-v4"
fallback = "claude-sonnet-v3.7"

[pgvector]
url = "postgres://localhost:5432/chatbot"

[data]
document_dir = "./data/faq_docs"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert_eq!(config.embedding.provider, "bedrock-cohere");
        assert_eq!(
            config.embedding.model.as_ref().unwrap(),
            "cohere.embed-multilingual-v3"
        );
        assert_eq!(config.embedding.aws_region.as_ref().unwrap(), "us-east-1");
    }
}
