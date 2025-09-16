//! Integration tests for database migrations
use std::time::Duration;
use testcontainers::core::WaitFor;
use testcontainers::{clients, GenericImage};
use tokio::time::sleep;
use tokio_postgres::NoTls;
use vector_store::run_migrations;

/// Helper function to wait for database to be ready with retries
async fn wait_for_database_ready(
    db_url: &str,
    max_retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    for attempt in 1..=max_retries {
        match tokio_postgres::connect(db_url, tokio_postgres::NoTls).await {
            Ok((client, connection)) => {
                // Spawn connection handler
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("Connection error: {}", e);
                    }
                });

                // Test a simple query to ensure database is fully ready
                match client.query_one("SELECT 1", &[]).await {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        eprintln!("Database query failed on attempt {}: {}", attempt, e);
                        if attempt == max_retries {
                            return Err(e.into());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Database connection failed on attempt {}: {}", attempt, e);
                if attempt == max_retries {
                    return Err(e.into());
                }
            }
        }

        // Wait before retrying
        sleep(Duration::from_millis(500 * attempt as u64)).await;
    }

    Err("Max retries exceeded".into())
}

#[tokio::test]
async fn should_apply_migrations_in_order() {
    // Create a new instance of the Docker client
    let docker = clients::Cli::default();
    let postgres_image = GenericImage::new("pgvector/pgvector", "pg16")
        .with_env_var("POSTGRES_DB", "test_chatbot")
        .with_env_var("POSTGRES_USER", "test_user")
        .with_env_var("POSTGRES_PASSWORD", "test_password")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ));

    let container = docker.run(postgres_image);
    let host_port = container.get_host_port_ipv4(5432);

    let db_url = format!(
        "postgresql://test_user:test_password@localhost:{}/test_chatbot", // pragma: allowlist secret
        host_port
    );

    // Wait for database to be fully ready
    wait_for_database_ready(&db_url, 10)
        .await
        .expect("Database should be ready within timeout");

    // Run migrations
    let result = run_migrations(&db_url).await;
    assert!(result.is_ok(), "Failed to apply migrations: {:?}", result);

    match result {
        Ok(_) => {
            // Success case - check if we can connect and verify table exists
            let (client, connection) = tokio_postgres::connect(&db_url, NoTls)
                .await
                .expect("Failed to connect to test database");

            // Spawn the connection handler
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            // Check if documents table exists
            let exists = client
                .query_one(
                    "SELECT EXISTS (
                            SELECT FROM pg_tables
                            WHERE schemaname = 'public'
                            AND tablename = 'documents'
                        )",
                    &[],
                )
                .await
                .expect("Failed to check table existence");

            let table_exists: bool = exists.get(0);
            assert!(
                table_exists,
                "Documents table should exist after migrations"
            );
        }
        Err(e) => {
            panic!("Failed to run migrations: {:?}", e);
        }
    }
}
