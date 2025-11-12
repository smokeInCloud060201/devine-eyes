use sea_orm::{Database, DatabaseConnection};
use anyhow::Result;

pub async fn create_connection(database_url: &str) -> Result<DatabaseConnection> {
    let db = Database::connect(database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    Ok(db)
}

// Note: Database migrations should be run separately using the migration CLI:
// cargo run --bin migration up
// Or use: cd migrations && cargo run -- up

