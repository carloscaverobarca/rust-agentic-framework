use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInput {
    pub name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

impl ToolInput {
    pub fn new(name: String) -> Self {
        Self {
            name,
            arguments: HashMap::new(),
        }
    }

    pub fn with_argument<T: Serialize>(mut self, key: &str, value: T) -> Result<Self> {
        let json_value = serde_json::to_value(value)?;
        self.arguments.insert(key.to_string(), json_value);
        Ok(self)
    }

    pub fn get_argument<T>(&self, key: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self
            .arguments
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Argument '{}' not found", key))?;

        let result: T = serde_json::from_value(value.clone())?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolOutput {
    pub success: bool,
    pub result: serde_json::Value,
    pub error_message: Option<String>,
}

impl ToolOutput {
    pub fn success<T: Serialize>(result: T) -> Result<Self> {
        Ok(Self {
            success: true,
            result: serde_json::to_value(result)?,
            error_message: None,
        })
    }

    pub fn error<T: Serialize>(error_message: String, partial_result: Option<T>) -> Result<Self> {
        let result = match partial_result {
            Some(data) => serde_json::to_value(data)?,
            None => serde_json::Value::Null,
        };

        Ok(Self {
            success: false,
            result,
            error_message: Some(error_message),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    pub tool_name: String,
    pub message: String,
    pub recoverable: bool,
}

impl ToolError {
    pub fn new(tool_name: String, message: String, recoverable: bool) -> Self {
        Self {
            tool_name,
            message,
            recoverable,
        }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tool '{}' error: {}", self.tool_name, self.message)
    }
}

impl std::error::Error for ToolError {}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;

    async fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError>;

    fn validate_input(&self, input: &ToolInput) -> Result<(), ToolError> {
        if input.name != self.name() {
            return Err(ToolError::new(
                self.name().to_string(),
                format!("Expected tool '{}', got '{}'", self.name(), input.name),
                false,
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_create_tool_input() {
        let input = ToolInput::new("test_tool".to_string());
        assert_eq!(input.name, "test_tool");
        assert!(input.arguments.is_empty());
    }

    #[test]
    fn should_add_arguments_to_tool_input() {
        let input = ToolInput::new("test_tool".to_string())
            .with_argument("file_path", "/test/path")
            .unwrap()
            .with_argument("count", 42)
            .unwrap();

        assert_eq!(input.arguments.len(), 2);
        assert_eq!(input.arguments["file_path"], json!("/test/path"));
        assert_eq!(input.arguments["count"], json!(42));
    }

    #[test]
    fn should_retrieve_arguments_from_tool_input() {
        let input = ToolInput::new("test_tool".to_string())
            .with_argument("file_path", "/test/path")
            .unwrap()
            .with_argument("count", 42)
            .unwrap();

        let file_path: String = input.get_argument("file_path").unwrap();
        let count: i32 = input.get_argument("count").unwrap();

        assert_eq!(file_path, "/test/path");
        assert_eq!(count, 42);
    }

    #[test]
    fn should_fail_to_get_nonexistent_argument() {
        let input = ToolInput::new("test_tool".to_string());
        let result: Result<String> = input.get_argument("missing");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn should_create_success_tool_output() {
        let output = ToolOutput::success("test result").unwrap();

        assert!(output.success);
        assert_eq!(output.result, json!("test result"));
        assert!(output.error_message.is_none());
    }

    #[test]
    fn should_create_error_tool_output() {
        let output =
            ToolOutput::error("Something went wrong".to_string(), Some("partial")).unwrap();

        assert!(!output.success);
        assert_eq!(output.result, json!("partial"));
        assert_eq!(
            output.error_message,
            Some("Something went wrong".to_string())
        );
    }

    #[test]
    fn should_create_error_tool_output_without_partial_result() {
        let output: ToolOutput =
            ToolOutput::error("Error occurred".to_string(), None::<String>).unwrap();

        assert!(!output.success);
        assert_eq!(output.result, serde_json::Value::Null);
        assert_eq!(output.error_message, Some("Error occurred".to_string()));
    }

    #[test]
    fn should_create_tool_error() {
        let error = ToolError::new(
            "test_tool".to_string(),
            "Test error message".to_string(),
            true,
        );

        assert_eq!(error.tool_name, "test_tool");
        assert_eq!(error.message, "Test error message");
        assert!(error.recoverable);
        assert_eq!(
            error.to_string(),
            "Tool 'test_tool' error: Test error message"
        );
    }

    #[test]
    fn should_serialize_and_deserialize_tool_input() {
        let input = ToolInput::new("test_tool".to_string())
            .with_argument("file_path", "/test/path")
            .unwrap();

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: ToolInput = serde_json::from_str(&json).unwrap();

        assert_eq!(input, deserialized);
    }

    #[test]
    fn should_serialize_and_deserialize_tool_output() {
        let output = ToolOutput::success("test result").unwrap();

        let json = serde_json::to_string(&output).unwrap();
        let deserialized: ToolOutput = serde_json::from_str(&json).unwrap();

        assert_eq!(output, deserialized);
    }

    // Mock tool for testing trait
    struct MockTool {
        name: String,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn parameters(&self) -> serde_json::Value {
            json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            })
        }

        async fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
            self.validate_input(&input)?;
            let text: String = input
                .get_argument("input")
                .map_err(|e| ToolError::new(self.name().to_string(), e.to_string(), true))?;

            ToolOutput::success(format!("Processed: {}", text))
                .map_err(|e| ToolError::new(self.name().to_string(), e.to_string(), false))
        }
    }

    #[tokio::test]
    async fn should_execute_mock_tool_successfully() {
        let tool = MockTool::new("mock_tool");
        let input = ToolInput::new("mock_tool".to_string())
            .with_argument("input", "test data")
            .unwrap();

        let result = tool.execute(input).await.unwrap();

        assert!(result.success);
        assert_eq!(result.result, json!("Processed: test data"));
        assert!(result.error_message.is_none());
    }

    #[tokio::test]
    async fn should_validate_tool_name_mismatch() {
        let tool = MockTool::new("mock_tool");
        let input = ToolInput::new("wrong_tool".to_string());

        let result = tool.execute(input).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.tool_name, "mock_tool");
        assert!(error
            .message
            .contains("Expected tool 'mock_tool', got 'wrong_tool'"));
    }

    #[tokio::test]
    async fn should_handle_missing_required_argument() {
        let tool = MockTool::new("mock_tool");
        let input = ToolInput::new("mock_tool".to_string());

        let result = tool.execute(input).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.tool_name, "mock_tool");
        assert!(error.recoverable);
    }
}
