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
