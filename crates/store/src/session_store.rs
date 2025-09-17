use crate::models::{Message, SessionData};
use anyhow::{Context, Result};
use redis::{Client, Commands, Connection};
use std::time::Duration;
use uuid::Uuid;

pub struct RedisSessionStore {
    client: Client,
    ttl: Duration,
}

impl RedisSessionStore {
    pub fn new(redis_url: &str, ttl: Duration) -> Result<Self> {
        let client = Client::open(redis_url).context("Failed to create Redis client")?;

        // Test the connection to ensure it's valid
        let _conn = client
            .get_connection()
            .context("Failed to connect to Redis")?;

        Ok(Self { client, ttl })
    }

    pub async fn get(&self, session_id: &Uuid) -> Result<Vec<Message>> {
        let mut conn = self.get_connection()?;
        let key = format!("session:{}", session_id);

        let data: Option<String> = conn.get(&key).context("Failed to get session from Redis")?;

        match data {
            Some(json) => {
                let session_data: SessionData =
                    serde_json::from_str(&json).context("Failed to deserialize session data")?;
                Ok(session_data.messages)
            }
            None => Ok(Vec::new()),
        }
    }

    pub async fn append(&self, session_id: &Uuid, message: Message) -> Result<()> {
        let mut conn = self.get_connection()?;
        let key = format!("session:{}", session_id);

        // Get existing session or create new one
        let mut session_data = self.get_session_data(&mut conn, &key)?;
        session_data.messages.push(message);
        session_data.last_accessed = chrono::Utc::now();

        // Store back to Redis with TTL
        let json =
            serde_json::to_string(&session_data).context("Failed to serialize session data")?;

        conn.set_ex::<_, _, ()>(&key, json, self.ttl.as_secs())
            .context("Failed to store session in Redis")?;

        Ok(())
    }

    pub async fn exists(&self, session_id: &Uuid) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let key = format!("session:{}", session_id);

        let exists: bool = conn
            .exists(&key)
            .context("Failed to check session existence")?;

        Ok(exists)
    }

    pub async fn delete(&self, session_id: &Uuid) -> Result<()> {
        let mut conn = self.get_connection()?;
        let key = format!("session:{}", session_id);

        conn.del::<_, ()>(&key)
            .context("Failed to delete session from Redis")?;

        Ok(())
    }

    pub async fn extend_ttl(&self, session_id: &Uuid) -> Result<()> {
        let mut conn = self.get_connection()?;
        let key = format!("session:{}", session_id);

        conn.expire::<_, ()>(&key, self.ttl.as_secs() as i64)
            .context("Failed to extend session TTL")?;

        Ok(())
    }

    // Private helper methods
    fn get_connection(&self) -> Result<Connection> {
        self.client
            .get_connection()
            .context("Failed to get Redis connection")
    }

    fn get_session_data(&self, conn: &mut Connection, key: &str) -> Result<SessionData> {
        let data: Option<String> = conn.get(key).context("Failed to get session data")?;

        match data {
            Some(json) => {
                let mut session_data: SessionData =
                    serde_json::from_str(&json).context("Failed to deserialize session data")?;
                session_data.last_accessed = chrono::Utc::now();
                Ok(session_data)
            }
            None => Ok(SessionData::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Role;

    #[tokio::test]
    async fn should_create_redis_session_store() {
        // This test requires a Redis instance running
        // For now, we'll just test that the struct can be created
        let result = RedisSessionStore::new("redis://localhost:6379", Duration::from_secs(3600));
        // We expect this to fail in CI without Redis, but the struct creation should work
        match result {
            Ok(_) => {
                // Redis is available, test basic operations
                let store = result.unwrap();
                let session_id = Uuid::new_v4();

                // Test getting empty session
                let messages = store.get(&session_id).await.unwrap();
                assert!(messages.is_empty());

                // Test appending message
                let message = Message {
                    role: Role::User,
                    content: "Hello".to_string(),
                    name: None,
                };

                store.append(&session_id, message.clone()).await.unwrap();

                // Test getting session with message
                let messages = store.get(&session_id).await.unwrap();
                assert_eq!(messages.len(), 1);
                assert_eq!(messages[0], message);

                // Test session exists
                let exists = store.exists(&session_id).await.unwrap();
                assert!(exists);

                // Test delete
                store.delete(&session_id).await.unwrap();
                let exists = store.exists(&session_id).await.unwrap();
                assert!(!exists);
            }
            Err(_) => {
                // Redis not available, skip integration tests
                println!("Redis not available, skipping integration tests");
            }
        }
    }

    #[test]
    fn should_handle_redis_connection_error() {
        let result = RedisSessionStore::new("redis://invalid:6379", Duration::from_secs(3600));
        assert!(result.is_err());
    }
}
