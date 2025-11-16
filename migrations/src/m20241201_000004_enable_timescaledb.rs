use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Enable TimescaleDB extension
        // Note: This requires superuser privileges
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to enable TimescaleDB extension: {}", e)))?;

        log::info!("TimescaleDB extension enabled");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Note: Dropping extension requires dropping all hypertables first
        // This is handled by the down migration of hypertable conversion
        Ok(())
    }
}

