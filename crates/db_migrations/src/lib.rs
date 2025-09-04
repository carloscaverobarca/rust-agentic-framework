use anyhow::{Context, Result};
use tokio_postgres::NoTls;
use tracing::info;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub async fn run_migrations(database_url: &str) -> Result<()> {
    info!("Running database migrations...");

    // Create database connection with a timeout
    let connect_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio_postgres::connect(database_url, NoTls),
    )
    .await
    .context("Database connection timed out")?;

    let (mut client, connection) = connect_result.context("Failed to connect to database")?;

    // Spawn connection handling future to a separate task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("Connection error: {}", e);
        }
    });

    // Run migrations
    let report = embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .context("Failed to run migrations")?;

    for migration in report.applied_migrations() {
        info!("Applied migration version {}", migration.version());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_run_migrations_successfully() {
        // Use the test database URL that matches the docker-compose.yml configuration
        let database_url = "postgresql://test_user:test_password@localhost:5432/test_chatbot";
        let result = run_migrations(database_url).await;

        match result {
            Ok(_) => {
                // Success case - check if we can connect and verify table exists
                let (client, connection) = tokio_postgres::connect(database_url, NoTls)
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
}
