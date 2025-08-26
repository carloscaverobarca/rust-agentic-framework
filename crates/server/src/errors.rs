use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Embedding service error: {0}")]
    EmbeddingError(String),

    #[error("Tool execution error: {0}")]
    ToolError(String),

    #[error("LLM service error: {0}")]
    LlmError(String),

    #[error("Database connection error: {0}")]
    DatabaseError(String),

    #[error("Vector store error: {0}")]
    VectorStoreError(String),

    #[error("Session management error: {0}")]
    SessionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid request: {0}")]
    ValidationError(String),
}

impl AgentError {
    /// Returns the appropriate HTTP status code for this error
    pub fn http_status_code(&self) -> u16 {
        match self {
            AgentError::EmbeddingError(_) => 500,   // Internal Server Error
            AgentError::ToolError(_) => 400,        // Bad Request (usually file not found)
            AgentError::LlmError(_) => 503,         // Service Unavailable (can retry)
            AgentError::DatabaseError(_) => 500,    // Internal Server Error
            AgentError::VectorStoreError(_) => 500, // Internal Server Error
            AgentError::SessionError(_) => 422,     // Unprocessable Entity
            AgentError::ConfigError(_) => 500,      // Internal Server Error
            AgentError::ValidationError(_) => 400,  // Bad Request
        }
    }

    /// Returns true if the error is potentially recoverable with a retry
    pub fn is_retryable(&self) -> bool {
        match self {
            AgentError::EmbeddingError(_) => true, // Cohere API might be temporarily down
            AgentError::ToolError(_) => false,     // File not found won't fix itself
            AgentError::LlmError(_) => true,       // Bedrock timeout can be retried
            AgentError::DatabaseError(_) => true,  // Connection can be re-established
            AgentError::VectorStoreError(_) => true, // DB issue, can retry
            AgentError::SessionError(_) => false,  // Session issues are not retryable
            AgentError::ConfigError(_) => false,   // Config errors need manual fix
            AgentError::ValidationError(_) => false, // Input validation errors can't be retried
        }
    }

    /// Converts error to SSE event format
    pub fn to_sse_event_data(&self) -> String {
        format!(
            "{{\"error\": \"{}\", \"retryable\": {}}}",
            self,
            self.is_retryable()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_correct_http_status_codes() {
        assert_eq!(
            AgentError::EmbeddingError("test".to_string()).http_status_code(),
            500
        );
        assert_eq!(
            AgentError::ToolError("test".to_string()).http_status_code(),
            400
        );
        assert_eq!(
            AgentError::LlmError("test".to_string()).http_status_code(),
            503
        );
        assert_eq!(
            AgentError::DatabaseError("test".to_string()).http_status_code(),
            500
        );
        assert_eq!(
            AgentError::VectorStoreError("test".to_string()).http_status_code(),
            500
        );
        assert_eq!(
            AgentError::SessionError("test".to_string()).http_status_code(),
            422
        );
        assert_eq!(
            AgentError::ConfigError("test".to_string()).http_status_code(),
            500
        );
        assert_eq!(
            AgentError::ValidationError("test".to_string()).http_status_code(),
            400
        );
    }

    #[test]
    fn should_return_correct_retryable_flags() {
        assert!(AgentError::EmbeddingError("test".to_string()).is_retryable());
        assert!(!AgentError::ToolError("test".to_string()).is_retryable());
        assert!(AgentError::LlmError("test".to_string()).is_retryable());
        assert!(AgentError::DatabaseError("test".to_string()).is_retryable());
        assert!(AgentError::VectorStoreError("test".to_string()).is_retryable());
        assert!(!AgentError::SessionError("test".to_string()).is_retryable());
        assert!(!AgentError::ConfigError("test".to_string()).is_retryable());
        assert!(!AgentError::ValidationError("test".to_string()).is_retryable());
    }

    #[test]
    fn should_format_sse_event_data_correctly() {
        let error = AgentError::EmbeddingError("Cohere API failed".to_string());
        let sse_data = error.to_sse_event_data();

        assert!(sse_data.contains("Embedding service error: Cohere API failed"));
        assert!(sse_data.contains("\"retryable\": true"));

        let non_retryable_error = AgentError::ToolError("File not found".to_string());
        let sse_data_non_retryable = non_retryable_error.to_sse_event_data();

        assert!(sse_data_non_retryable.contains("Tool execution error: File not found"));
        assert!(sse_data_non_retryable.contains("\"retryable\": false"));
    }
}
