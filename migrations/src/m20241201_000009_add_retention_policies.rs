use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Add retention policy for container_stats (keep raw data for 7 days)
        conn.execute_unprepared(
            r#"
            SELECT add_retention_policy('container_stats', 
                INTERVAL '7 days',
                if_not_exists => TRUE);
            "#,
        )
        .await
        .ok(); // Ignore error if policy already exists

        // Add retention policy for container_logs (keep raw data for 3 days)
        conn.execute_unprepared(
            r#"
            SELECT add_retention_policy('container_logs', 
                INTERVAL '3 days',
                if_not_exists => TRUE);
            "#,
        )
        .await
        .ok();

        // Add retention policy for image_versions (keep for 30 days)
        conn.execute_unprepared(
            r#"
            SELECT add_retention_policy('image_versions', 
                INTERVAL '30 days',
                if_not_exists => TRUE);
            "#,
        )
        .await
        .ok();

        // Enable compression for container_stats (compress data older than 1 day)
        conn.execute_unprepared(
            r#"
            ALTER TABLE container_stats SET (
                timescaledb.compress,
                timescaledb.compress_segmentby = 'container_id'
            );
            SELECT add_compression_policy('container_stats', 
                INTERVAL '1 day',
                if_not_exists => TRUE);
            "#,
        )
        .await
        .ok();

        // Enable compression for container_logs
        conn.execute_unprepared(
            r#"
            ALTER TABLE container_logs SET (
                timescaledb.compress,
                timescaledb.compress_segmentby = 'container_id'
            );
            SELECT add_compression_policy('container_logs', 
                INTERVAL '1 day',
                if_not_exists => TRUE);
            "#,
        )
        .await
        .ok();

        log::info!("Added retention and compression policies");

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Note: Removing retention policies requires manual intervention
        // as TimescaleDB doesn't provide a simple DROP command
        // This is intentional - retention policies should be managed carefully
        Ok(())
    }
}

