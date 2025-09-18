use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream};
use serde_json::Value;
use std::convert::Infallible;
use std::time::Duration;

pub fn create_sse_stream(
    events: Vec<Event>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send> {
    let stream = stream::iter(events.into_iter().map(Ok));
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive-text"),
    )
}

pub fn create_tool_usage_event(
    tool_name: &str,
    args: Value,
    duration_ms: u64,
    result: &str,
) -> Event {
    let data = serde_json::json!({
        "tool": tool_name,
        "args": args,
        "duration_ms": duration_ms,
        "result": result
    });

    Event::default().event("tool_usage").data(data.to_string())
}

pub fn create_assistant_output_event(content: &str) -> Event {
    let data = serde_json::json!({
        "content": content
    });

    Event::default()
        .event("assistant_output")
        .data(data.to_string())
}

pub fn create_streaming_content_event(content: &str) -> Event {
    let data = serde_json::json!({
        "content": content,
        "type": "delta"
    });

    Event::default()
        .event("content_delta")
        .data(data.to_string())
}

pub fn create_stream_start_event() -> Event {
    Event::default().event("stream_start").data("{}")
}

pub fn create_stream_end_event() -> Event {
    Event::default().event("stream_end").data("{}")
}

pub fn create_error_event(error: &crate::errors::AgentError) -> Event {
    let data = serde_json::json!({
        "error": error.to_string(),
        "retryable": error.is_retryable(),
        "http_status": error.http_status_code()
    });

    Event::default().event("error_event").data(data.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use serde_json::json;
    use tower::ServiceExt;

    async fn test_sse_endpoint() -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send> {
        let events = vec![
            create_tool_usage_event(
                "file_summarizer",
                json!({"file_path": "test.txt"}),
                123,
                "Summary content",
            ),
            create_assistant_output_event("Hello from assistant"),
        ];

        create_sse_stream(events)
    }

    #[tokio::test]
    async fn should_create_tool_usage_event() {
        let event = create_tool_usage_event(
            "file_summarizer",
            json!({"file_path": "test.txt"}),
            123,
            "Test summary",
        );

        // Convert to debug string to verify content
        let event_str = format!("{:?}", event);
        assert!(event_str.contains("tool_usage"));
        assert!(event_str.contains("file_summarizer"));
        assert!(event_str.contains("test.txt"));
        assert!(event_str.contains("Test summary"));
    }

    #[tokio::test]
    async fn should_create_assistant_output_event() {
        let event = create_assistant_output_event("Hello world");

        // Convert to debug string to verify content
        let event_str = format!("{:?}", event);
        assert!(event_str.contains("assistant_output"));
        assert!(event_str.contains("Hello world"));
    }

    #[tokio::test]
    async fn should_create_sse_endpoint_with_multiple_events() {
        let app = Router::new().route("/sse", get(test_sse_endpoint));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sse")
                    .header("Accept", "text/event-stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
        assert_eq!(response.headers().get("cache-control").unwrap(), "no-cache");

        // Test that we can collect the stream
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let content = String::from_utf8(bytes.to_vec()).unwrap();

        // Should contain our events
        assert!(content.contains("event: tool_usage"));
        assert!(content.contains("event: assistant_output"));
        assert!(content.contains("file_summarizer"));
        assert!(content.contains("Hello from assistant"));
    }

    #[tokio::test]
    async fn should_handle_empty_event_stream() {
        let events: Vec<Event> = vec![];
        let _stream = create_sse_stream(events);

        // Verify the stream can be created without errors
        // This test just ensures empty streams don't panic
    }

    #[tokio::test]
    async fn should_create_error_event() {
        use crate::errors::AgentError;

        let error = AgentError::EmbeddingError("Embedding API timeout".to_string());
        let event = create_error_event(&error);

        // Convert to debug string to verify content
        let event_str = format!("{:?}", event);
        assert!(event_str.contains("error_event"));
        assert!(event_str.contains("Embedding service error"));
        assert!(event_str.contains("Embedding API timeout"));
        assert!(event_str.contains("retryable"));
        assert!(event_str.contains("500")); // HTTP status code
    }

    #[tokio::test]
    async fn should_create_error_event_with_different_error_types() {
        use crate::errors::AgentError;

        // Test tool error (non-retryable, 400 status)
        let tool_error = AgentError::ToolError("File not found".to_string());
        let event = create_error_event(&tool_error);
        let event_str = format!("{:?}", event);
        assert!(event_str.contains("Tool execution error"));
        assert!(event_str.contains("400"));

        // Test LLM error (retryable, 503 status)
        let llm_error = AgentError::LlmError("Bedrock timeout".to_string());
        let event = create_error_event(&llm_error);
        let event_str = format!("{:?}", event);
        assert!(event_str.contains("LLM service error"));
        assert!(event_str.contains("503"));
    }
}
