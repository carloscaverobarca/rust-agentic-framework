use crate::tool::{Tool, ToolError, ToolInput, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tokio::fs;

pub struct FileSummarizerTool {
    max_file_size: u64,
    allowed_extensions: Vec<String>,
}

impl FileSummarizerTool {
    pub fn new() -> Self {
        Self {
            max_file_size: 1024 * 1024, // 1MB default limit
            allowed_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "cfg".to_string(),
                "conf".to_string(),
            ],
        }
    }

    pub fn with_max_file_size(mut self, size_bytes: u64) -> Self {
        self.max_file_size = size_bytes;
        self
    }

    pub fn with_allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = extensions;
        self
    }

    fn is_allowed_file(&self, file_path: &str) -> bool {
        let path = Path::new(file_path);

        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.allowed_extensions.contains(&ext_str.to_lowercase());
            }
        }

        false
    }

    async fn read_file_content(&self, file_path: &str) -> Result<String, ToolError> {
        // Check if file exists
        if !Path::new(file_path).exists() {
            return Err(ToolError::new(
                self.name().to_string(),
                format!("File not found: {file_path}"),
                false,
            ));
        }

        // Check file extension
        if !self.is_allowed_file(file_path) {
            return Err(ToolError::new(
                self.name().to_string(),
                format!("File type not allowed: {file_path}"),
                false,
            ));
        }

        // Check file size
        let metadata = fs::metadata(file_path).await.map_err(|e| {
            ToolError::new(
                self.name().to_string(),
                format!("Failed to read file metadata: {e}"),
                true,
            )
        })?;

        if metadata.len() > self.max_file_size {
            return Err(ToolError::new(
                self.name().to_string(),
                format!(
                    "File too large: {} bytes (max: {} bytes)",
                    metadata.len(),
                    self.max_file_size
                ),
                false,
            ));
        }

        // Read file content
        let content = fs::read_to_string(file_path).await.map_err(|e| {
            ToolError::new(
                self.name().to_string(),
                format!("Failed to read file: {e}"),
                true,
            )
        })?;

        Ok(content)
    }

    fn summarize_content(&self, content: &str, file_path: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        let total_chars = content.chars().count();
        let total_words = content.split_whitespace().count();

        // Extract first few lines as preview
        let preview_lines = 5;
        let preview: Vec<&str> = lines.iter().take(preview_lines).cloned().collect();

        // Basic structure analysis
        let mut structure_info = Vec::new();

        // Count common programming constructs
        if file_path.ends_with(".rs") {
            let fn_count = content.matches("fn ").count();
            let struct_count = content.matches("struct ").count();
            let impl_count = content.matches("impl ").count();

            if fn_count > 0 {
                structure_info.push(format!("Functions: {fn_count}"));
            }
            if struct_count > 0 {
                structure_info.push(format!("Structs: {struct_count}"));
            }
            if impl_count > 0 {
                structure_info.push(format!("Implementations: {impl_count}"));
            }
        } else if file_path.ends_with(".py") {
            let def_count = content.matches("def ").count();
            let class_count = content.matches("class ").count();
            let import_count =
                content.matches("import ").count() + content.matches("from ").count();

            if def_count > 0 {
                structure_info.push(format!("Functions: {def_count}"));
            }
            if class_count > 0 {
                structure_info.push(format!("Classes: {class_count}"));
            }
            if import_count > 0 {
                structure_info.push(format!("Imports: {import_count}"));
            }
        }

        format!(
            "File: {}\nSize: {} lines, {} words, {} characters\nStructure: {}\n\nPreview (first {} lines):\n{}",
            file_path,
            total_lines,
            total_words,
            total_chars,
            if structure_info.is_empty() { "Plain text".to_string() } else { structure_info.join(", ") },
            preview_lines,
            preview.join("\n")
        )
    }
}

impl Default for FileSummarizerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileSummarizerTool {
    fn name(&self) -> &str {
        "file_summarizer"
    }

    fn description(&self) -> &str {
        "Summarizes the content of a text file, providing basic statistics and a preview"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to summarize"
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        self.validate_input(&input)?;

        let file_path: String = input
            .get_argument("file_path")
            .map_err(|e| ToolError::new(self.name().to_string(), e.to_string(), true))?;

        let content = self.read_file_content(&file_path).await?;
        let summary = self.summarize_content(&content, &file_path);

        ToolOutput::success(summary)
            .map_err(|e| ToolError::new(self.name().to_string(), e.to_string(), false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    async fn create_test_file(content: &str, extension: &str) -> NamedTempFile {
        let mut file = tempfile::Builder::new()
            .suffix(&format!(".{}", extension))
            .tempfile()
            .unwrap();

        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn should_create_file_summarizer_with_defaults() {
        let tool = FileSummarizerTool::new();
        assert_eq!(tool.name(), "file_summarizer");
        assert_eq!(tool.max_file_size, 1024 * 1024);
        assert!(tool.allowed_extensions.contains(&"txt".to_string()));
        assert!(tool.allowed_extensions.contains(&"rs".to_string()));
    }

    #[test]
    fn should_customize_file_summarizer() {
        let tool = FileSummarizerTool::new()
            .with_max_file_size(512)
            .with_allowed_extensions(vec!["custom".to_string()]);

        assert_eq!(tool.max_file_size, 512);
        assert_eq!(tool.allowed_extensions, vec!["custom"]);
    }

    #[test]
    fn should_check_allowed_file_types() {
        let tool = FileSummarizerTool::new();

        assert!(tool.is_allowed_file("test.txt"));
        assert!(tool.is_allowed_file("script.rs"));
        assert!(tool.is_allowed_file("config.json"));
        assert!(!tool.is_allowed_file("image.png"));
        assert!(!tool.is_allowed_file("binary.exe"));
        assert!(!tool.is_allowed_file("file_without_extension"));
    }

    #[tokio::test]
    async fn should_summarize_rust_file() {
        let content = r#"
struct TestStruct {
    field: String,
}

impl TestStruct {
    fn new() -> Self {
        Self { field: String::new() }
    }
}

fn helper_function() {
    println!("Hello");
}
"#;

        let file = create_test_file(content, "rs").await;
        let tool = FileSummarizerTool::new();

        let input = ToolInput::new("file_summarizer".to_string())
            .with_argument("file_path", file.path().to_str().unwrap())
            .unwrap();

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);

        let summary = result.result.as_str().unwrap();
        assert!(summary.contains("Functions: 2"));
        assert!(summary.contains("Structs: 1"));
        assert!(summary.contains("Implementations: 1"));
    }

    #[tokio::test]
    async fn should_summarize_python_file() {
        let content = r#"
import os
from typing import List

class TestClass:
    def __init__(self):
        pass
    
    def method(self):
        return "test"

def function1():
    pass

def function2():
    return 42
"#;

        let file = create_test_file(content, "py").await;
        let tool = FileSummarizerTool::new();

        let input = ToolInput::new("file_summarizer".to_string())
            .with_argument("file_path", file.path().to_str().unwrap())
            .unwrap();

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);

        let summary = result.result.as_str().unwrap();
        assert!(summary.contains("Functions: 4")); // __init__, method, function1, function2
        assert!(summary.contains("Classes: 1"));
        assert!(summary.contains("Imports: 3")); // import os, from typing
    }

    #[tokio::test]
    async fn should_summarize_text_file() {
        let content = "This is a simple text file.\nWith multiple lines.\nAnd some words.";
        let file = create_test_file(content, "txt").await;
        let tool = FileSummarizerTool::new();

        let input = ToolInput::new("file_summarizer".to_string())
            .with_argument("file_path", file.path().to_str().unwrap())
            .unwrap();

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);

        let summary = result.result.as_str().unwrap();
        assert!(summary.contains("3 lines"));
        assert!(summary.contains("12 words"));
        assert!(summary.contains("Structure: Plain text"));
        assert!(summary.contains("This is a simple text file."));
    }

    #[tokio::test]
    async fn should_fail_for_nonexistent_file() {
        let tool = FileSummarizerTool::new();
        let input = ToolInput::new("file_summarizer".to_string())
            .with_argument("file_path", "/nonexistent/file.txt")
            .unwrap();

        let result = tool.execute(input).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.tool_name, "file_summarizer");
        assert!(error.message.contains("File not found"));
    }

    #[tokio::test]
    async fn should_fail_for_disallowed_file_type() {
        let file = create_test_file("binary content", "exe").await;
        let tool = FileSummarizerTool::new();

        let input = ToolInput::new("file_summarizer".to_string())
            .with_argument("file_path", file.path().to_str().unwrap())
            .unwrap();

        let result = tool.execute(input).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("File type not allowed"));
    }

    #[tokio::test]
    async fn should_fail_for_oversized_file() {
        let content = "x".repeat(1000); // Small content for the test
        let file = create_test_file(&content, "txt").await;

        let tool = FileSummarizerTool::new().with_max_file_size(100); // Very small limit

        let input = ToolInput::new("file_summarizer".to_string())
            .with_argument("file_path", file.path().to_str().unwrap())
            .unwrap();

        let result = tool.execute(input).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("File too large"));
    }

    #[tokio::test]
    async fn should_validate_tool_name() {
        let tool = FileSummarizerTool::new();
        let input = ToolInput::new("wrong_tool".to_string())
            .with_argument("file_path", "test.txt")
            .unwrap();

        let result = tool.execute(input).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("Expected tool 'file_summarizer'"));
    }

    #[tokio::test]
    async fn should_fail_with_missing_file_path() {
        let tool = FileSummarizerTool::new();
        let input = ToolInput::new("file_summarizer".to_string());

        let result = tool.execute(input).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("not found"));
        assert!(error.recoverable);
    }
}
