use crate::tool::{Tool, ToolError, ToolInput, ToolOutput};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

pub type BoxedTool = Box<dyn Tool>;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<BoxedTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: BoxedTool) -> Result<()> {
        let name = tool.name().to_string();

        if self.tools.contains_key(&name) {
            anyhow::bail!("Tool '{}' is already registered", name);
        }

        self.tools.insert(name, Arc::new(tool));
        Ok(())
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<BoxedTool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn get_tool_schema(&self, name: &str) -> Option<serde_json::Value> {
        self.tools.get(name).map(|tool| {
            serde_json::json!({
                "name": tool.name(),
                "description": tool.description(),
                "parameters": tool.parameters()
            })
        })
    }

    pub fn get_all_schemas(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.parameters()
                })
            })
            .collect()
    }

    pub async fn execute_tool(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let tool = self.get_tool(&input.name).ok_or_else(|| {
            ToolError::new(
                input.name.clone(),
                format!("Tool '{}' not found in registry", input.name),
                false,
            )
        })?;

        tool.execute(input).await
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn remove_tool(&mut self, name: &str) -> Option<Arc<BoxedTool>> {
        self.tools.remove(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::Tool;
    use async_trait::async_trait;
    use serde_json::json;

    // Mock tool for testing
    struct TestTool {
        name: String,
        should_fail: bool,
    }

    impl TestTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_fail: false,
            }
        }

        fn new_failing(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "A test tool"
        }

        fn parameters(&self) -> serde_json::Value {
            json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                }
            })
        }

        async fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
            self.validate_input(&input)?;

            if self.should_fail {
                return Err(ToolError::new(
                    self.name().to_string(),
                    "Simulated failure".to_string(),
                    true,
                ));
            }

            let message: String = input
                .get_argument("message")
                .map_err(|e| ToolError::new(self.name().to_string(), e.to_string(), true))?;

            ToolOutput::success(format!("Echo: {}", message))
                .map_err(|e| ToolError::new(self.name().to_string(), e.to_string(), false))
        }
    }

    #[test]
    fn should_create_empty_registry() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.tool_count(), 0);
        assert!(registry.list_tools().is_empty());
    }

    #[test]
    fn should_register_tool_successfully() {
        let mut registry = ToolRegistry::new();
        let tool = Box::new(TestTool::new("test_tool"));

        let result = registry.register(tool);
        assert!(result.is_ok());
        assert_eq!(registry.tool_count(), 1);
        assert!(registry.is_registered("test_tool"));
    }

    #[test]
    fn should_fail_to_register_duplicate_tool() {
        let mut registry = ToolRegistry::new();
        let tool1 = Box::new(TestTool::new("test_tool"));
        let tool2 = Box::new(TestTool::new("test_tool"));

        registry.register(tool1).unwrap();
        let result = registry.register(tool2);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already registered"));
        assert_eq!(registry.tool_count(), 1);
    }

    #[test]
    fn should_get_registered_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Box::new(TestTool::new("test_tool"));

        registry.register(tool).unwrap();

        let retrieved = registry.get_tool("test_tool");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test_tool");
    }

    #[test]
    fn should_return_none_for_unregistered_tool() {
        let registry = ToolRegistry::new();
        let retrieved = registry.get_tool("nonexistent_tool");
        assert!(retrieved.is_none());
    }

    #[test]
    fn should_list_all_registered_tools() {
        let mut registry = ToolRegistry::new();
        let tool1 = Box::new(TestTool::new("tool_1"));
        let tool2 = Box::new(TestTool::new("tool_2"));

        registry.register(tool1).unwrap();
        registry.register(tool2).unwrap();

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"tool_1".to_string()));
        assert!(tools.contains(&"tool_2".to_string()));
    }

    #[test]
    fn should_get_tool_schema() {
        let mut registry = ToolRegistry::new();
        let tool = Box::new(TestTool::new("test_tool"));

        registry.register(tool).unwrap();

        let schema = registry.get_tool_schema("test_tool").unwrap();
        assert_eq!(schema["name"], "test_tool");
        assert_eq!(schema["description"], "A test tool");
        assert!(schema["parameters"].is_object());
    }

    #[test]
    fn should_get_all_schemas() {
        let mut registry = ToolRegistry::new();
        let tool1 = Box::new(TestTool::new("tool_1"));
        let tool2 = Box::new(TestTool::new("tool_2"));

        registry.register(tool1).unwrap();
        registry.register(tool2).unwrap();

        let schemas = registry.get_all_schemas();
        assert_eq!(schemas.len(), 2);

        let names: Vec<&str> = schemas
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"tool_1"));
        assert!(names.contains(&"tool_2"));
    }

    #[tokio::test]
    async fn should_execute_registered_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Box::new(TestTool::new("test_tool"));

        registry.register(tool).unwrap();

        let input = ToolInput::new("test_tool".to_string())
            .with_argument("message", "hello world")
            .unwrap();

        let result = registry.execute_tool(input).await.unwrap();
        assert!(result.success);
        assert_eq!(result.result, json!("Echo: hello world"));
    }

    #[tokio::test]
    async fn should_fail_to_execute_unregistered_tool() {
        let registry = ToolRegistry::new();
        let input = ToolInput::new("nonexistent_tool".to_string());

        let result = registry.execute_tool(input).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.tool_name, "nonexistent_tool");
        assert!(error.message.contains("not found in registry"));
    }

    #[tokio::test]
    async fn should_handle_tool_execution_failure() {
        let mut registry = ToolRegistry::new();
        let tool = Box::new(TestTool::new_failing("failing_tool"));

        registry.register(tool).unwrap();

        let input = ToolInput::new("failing_tool".to_string())
            .with_argument("message", "test")
            .unwrap();

        let result = registry.execute_tool(input).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.tool_name, "failing_tool");
        assert!(error.message.contains("Simulated failure"));
        assert!(error.recoverable);
    }

    #[test]
    fn should_remove_tool_from_registry() {
        let mut registry = ToolRegistry::new();
        let tool = Box::new(TestTool::new("test_tool"));

        registry.register(tool).unwrap();
        assert!(registry.is_registered("test_tool"));

        let removed = registry.remove_tool("test_tool");
        assert!(removed.is_some());
        assert!(!registry.is_registered("test_tool"));
        assert_eq!(registry.tool_count(), 0);
    }

    #[test]
    fn should_return_none_when_removing_unregistered_tool() {
        let mut registry = ToolRegistry::new();
        let removed = registry.remove_tool("nonexistent_tool");
        assert!(removed.is_none());
    }
}
