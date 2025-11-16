use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Create composite index for container_stats (container_id, timestamp)
        // This is optimized for time-range queries per container
        conn.execute_unprepared(
            r#"
            CREATE INDEX IF NOT EXISTS idx_container_stats_container_timestamp 
            ON container_stats (container_id, timestamp DESC);
            "#,
        )
        .await
        .ok();

        // Create composite index for container_logs
        conn.execute_unprepared(
            r#"
            CREATE INDEX IF NOT EXISTS idx_container_logs_container_timestamp 
            ON container_logs (container_id, timestamp DESC);
            "#,
        )
        .await
        .ok();

        // Create composite index for image_versions
        conn.execute_unprepared(
            r#"
            CREATE INDEX IF NOT EXISTS idx_image_versions_image_timestamp 
            ON image_versions (image_id, timestamp DESC);
            "#,
        )
        .await
        .ok();

        // Create continuous aggregate for hourly container stats
        conn.execute_unprepared(
            r#"
            CREATE MATERIALIZED VIEW IF NOT EXISTS container_stats_hourly
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('1 hour', timestamp) AS bucket,
                container_id,
                container_name,
                AVG(cpu_usage_percent) AS avg_cpu_usage_percent,
                MAX(cpu_usage_percent) AS max_cpu_usage_percent,
                MIN(cpu_usage_percent) AS min_cpu_usage_percent,
                AVG(memory_usage_bytes) AS avg_memory_usage_bytes,
                MAX(memory_usage_bytes) AS max_memory_usage_bytes,
                AVG(memory_usage_percent) AS avg_memory_usage_percent,
                MAX(memory_usage_percent) AS max_memory_usage_percent,
                SUM(network_rx_bytes) AS total_network_rx_bytes,
                SUM(network_tx_bytes) AS total_network_tx_bytes,
                SUM(block_read_bytes) AS total_block_read_bytes,
                SUM(block_write_bytes) AS total_block_write_bytes
            FROM container_stats
            GROUP BY bucket, container_id, container_name
            WITH NO DATA;
            "#,
        )
        .await
        .map_err(|e| DbErr::Custom(format!("Failed to create continuous aggregate: {}", e)))?;

        // Add refresh policy for hourly aggregate
        conn.execute_unprepared(
            r#"
            SELECT add_continuous_aggregate_policy('container_stats_hourly',
                start_offset => INTERVAL '3 hours',
                end_offset => INTERVAL '1 hour',
                schedule_interval => INTERVAL '1 hour',
                if_not_exists => TRUE);
            "#,
        )
        .await
        .ok(); // Ignore error if policy already exists

        log::info!("Created continuous aggregates and indexes");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Drop continuous aggregate
        conn.execute_unprepared(
            "DROP MATERIALIZED VIEW IF EXISTS container_stats_hourly CASCADE;",
        )
        .await
        .ok();

        // Drop indexes
        conn.execute_unprepared(
            "DROP INDEX IF EXISTS idx_container_stats_container_timestamp;",
        )
        .await
        .ok();

        conn.execute_unprepared(
            "DROP INDEX IF EXISTS idx_container_logs_container_timestamp;",
        )
        .await
        .ok();

        conn.execute_unprepared(
            "DROP INDEX IF EXISTS idx_image_versions_image_timestamp;",
        )
        .await
        .ok();

        Ok(())
    }
}

