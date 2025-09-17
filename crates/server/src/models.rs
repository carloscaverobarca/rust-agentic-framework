use serde::{Deserialize, Serialize};
use store::Message;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictStreamRequest {
    pub session_id: Uuid,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictStreamResponse {
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use store::Role;

    #[test]
    fn should_serialize_predict_stream_request() {
        let request = PredictStreamRequest {
            session_id: Uuid::new_v4(),
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
                name: None,
            }],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("session_id"));
        assert!(json.contains("messages"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn should_deserialize_predict_stream_request() {
        let session_id = Uuid::new_v4();
        let json = format!(
            r#"{{
            "session_id": "{}",
            "messages": [
                {{
                    "role": "User",
                    "content": "How do I submit expenses?",
                    "name": null
                }}
            ]
        }}"#,
            session_id
        );

        let request: PredictStreamRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.session_id, session_id);
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, Role::User);
        assert_eq!(request.messages[0].content, "How do I submit expenses?");
        assert_eq!(request.messages[0].name, None);
    }

    #[test]
    fn should_serialize_predict_stream_response() {
        let response = PredictStreamResponse {
            status: "ok".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let expected = r#"{"status":"ok"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn should_deserialize_predict_stream_response() {
        let json = r#"{"status":"processing"}"#;
        let response: PredictStreamResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, "processing");
    }
}
