use crate::Message;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug)]
struct SessionData {
    messages: Vec<Message>,
    last_accessed: Instant,
}

#[derive(Debug, Default)]
pub struct SessionStore {
    sessions: HashMap<Uuid, SessionData>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn get(&self, session_id: &Uuid) -> Vec<Message> {
        self.sessions
            .get(session_id)
            .map(|data| data.messages.clone())
            .unwrap_or_default()
    }

    pub fn append(&mut self, session_id: &Uuid, message: Message) {
        let now = Instant::now();

        match self.sessions.get_mut(session_id) {
            Some(session_data) => {
                session_data.messages.push(message);
                session_data.last_accessed = now;
            }
            None => {
                let session_data = SessionData {
                    messages: vec![message],
                    last_accessed: now,
                };
                self.sessions.insert(*session_id, session_data);
            }
        }
    }

    pub fn gc(&mut self, ttl: Duration) {
        let now = Instant::now();
        self.sessions
            .retain(|_, session_data| now.duration_since(session_data.last_accessed) < ttl);
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, Role};

    #[tokio::test]
    async fn should_create_empty_session_store() {
        let store = SessionStore::new();
        assert_eq!(store.len(), 0);
    }

    #[tokio::test]
    async fn should_get_empty_messages_for_new_session() {
        let store = SessionStore::new();
        let session_id = Uuid::new_v4();

        let messages = store.get(&session_id);
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn should_append_message_to_session() {
        let mut store = SessionStore::new();
        let session_id = Uuid::new_v4();

        let message = Message {
            role: Role::User,
            content: "Hello".to_string(),
            name: None,
        };

        store.append(&session_id, message.clone());

        let messages = store.get(&session_id);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], message);
    }

    #[tokio::test]
    async fn should_append_multiple_messages_to_session() {
        let mut store = SessionStore::new();
        let session_id = Uuid::new_v4();

        let user_message = Message {
            role: Role::User,
            content: "Hello".to_string(),
            name: None,
        };

        let assistant_message = Message {
            role: Role::Assistant,
            content: "Hi there!".to_string(),
            name: None,
        };

        store.append(&session_id, user_message.clone());
        store.append(&session_id, assistant_message.clone());

        let messages = store.get(&session_id);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], user_message);
        assert_eq!(messages[1], assistant_message);
    }

    #[tokio::test]
    async fn should_keep_separate_sessions() {
        let mut store = SessionStore::new();
        let session1 = Uuid::new_v4();
        let session2 = Uuid::new_v4();

        let message1 = Message {
            role: Role::User,
            content: "Session 1".to_string(),
            name: None,
        };

        let message2 = Message {
            role: Role::User,
            content: "Session 2".to_string(),
            name: None,
        };

        store.append(&session1, message1.clone());
        store.append(&session2, message2.clone());

        let messages1 = store.get(&session1);
        let messages2 = store.get(&session2);

        assert_eq!(messages1.len(), 1);
        assert_eq!(messages2.len(), 1);
        assert_eq!(messages1[0], message1);
        assert_eq!(messages2[0], message2);
    }

    #[tokio::test]
    async fn should_cleanup_expired_sessions() {
        let mut store = SessionStore::new();
        let session_id = Uuid::new_v4();

        let message = Message {
            role: Role::User,
            content: "Test".to_string(),
            name: None,
        };

        store.append(&session_id, message);
        assert_eq!(store.len(), 1);

        // Simulate time passing by manually setting old timestamps
        // This test verifies the gc method removes expired sessions
        store.gc(Duration::from_millis(0)); // Everything should be expired

        assert_eq!(store.len(), 0);
    }

    #[tokio::test]
    async fn should_not_cleanup_recent_sessions() {
        let mut store = SessionStore::new();
        let session_id = Uuid::new_v4();

        let message = Message {
            role: Role::User,
            content: "Test".to_string(),
            name: None,
        };

        store.append(&session_id, message);
        assert_eq!(store.len(), 1);

        // With a long TTL, sessions should not be cleaned up
        store.gc(Duration::from_secs(3600)); // 1 hour TTL

        assert_eq!(store.len(), 1);
    }

    #[tokio::test]
    async fn should_return_session_count() {
        let mut store = SessionStore::new();
        assert_eq!(store.len(), 0);

        let session1 = Uuid::new_v4();
        let session2 = Uuid::new_v4();

        let message = Message {
            role: Role::User,
            content: "Test".to_string(),
            name: None,
        };

        store.append(&session1, message.clone());
        assert_eq!(store.len(), 1);

        store.append(&session2, message);
        assert_eq!(store.len(), 2);
    }
}
