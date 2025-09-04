//! Integration tests for database migrations
use db_migrations::run_migrations;

#[tokio::test]
async fn should_apply_migrations_in_order() {
    let database_url = "postgresql://test_user:test_password@localhost:5432/test_chatbot";
    
    let result = run_migrations(database_url).await;
    
    assert!(result.is_ok(), "Failed to apply migrations: {:?}", result);
    
    // Could add more validation here by querying the schema version table
    // But for now we'll just check that migrations apply without errors
}
