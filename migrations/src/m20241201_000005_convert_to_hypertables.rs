use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Convert container_stats to hypertable
        // TimescaleDB requires that unique constraints include the partition key
        // So we need to drop the primary key first, then recreate it with timestamp
        conn.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM timescaledb_information.hypertables 
                    WHERE hypertable_name = 'container_stats'
                ) THEN
                    -- Drop the primary key constraint if it exists
                    IF EXISTS (
                        SELECT 1 FROM pg_constraint 
                        WHERE conname = 'container_stats_pkey'
                    ) THEN
                        ALTER TABLE container_stats DROP CONSTRAINT container_stats_pkey;
                    END IF;
                    
                    -- Convert to hypertable
                    PERFORM create_hypertable('container_stats', 'timestamp', 
                        chunk_time_interval => INTERVAL '1 day',
                        if_not_exists => TRUE);
                    
                    -- Recreate primary key as composite (id, timestamp) to satisfy TimescaleDB
                    -- Note: This makes id non-unique by itself, but (id, timestamp) is unique
                    ALTER TABLE container_stats ADD CONSTRAINT container_stats_pkey 
                        PRIMARY KEY (id, timestamp);
                END IF;
            END $$;
            "#,
        )
        .await
        .map_err(|e| DbErr::Custom(format!("Failed to convert container_stats to hypertable: {}", e)))?;

        log::info!("Converted container_stats to hypertable");

        // Convert container_logs to hypertable
        conn.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM timescaledb_information.hypertables 
                    WHERE hypertable_name = 'container_logs'
                ) THEN
                    -- Drop the primary key constraint if it exists
                    IF EXISTS (
                        SELECT 1 FROM pg_constraint 
                        WHERE conname = 'container_logs_pkey'
                    ) THEN
                        ALTER TABLE container_logs DROP CONSTRAINT container_logs_pkey;
                    END IF;
                    
                    -- Convert to hypertable
                    PERFORM create_hypertable('container_logs', 'timestamp', 
                        chunk_time_interval => INTERVAL '1 day',
                        if_not_exists => TRUE);
                    
                    -- Recreate primary key as composite (id, timestamp) to satisfy TimescaleDB
                    ALTER TABLE container_logs ADD CONSTRAINT container_logs_pkey 
                        PRIMARY KEY (id, timestamp);
                END IF;
            END $$;
            "#,
        )
        .await
        .map_err(|e| DbErr::Custom(format!("Failed to convert container_logs to hypertable: {}", e)))?;

        log::info!("Converted container_logs to hypertable");

        // Note: http_requests table conversion is handled in migration 000010
        // after the table is created

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Drop hypertables (this will convert them back to regular tables)
        conn.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF EXISTS (
                    SELECT 1 FROM timescaledb_information.hypertables 
                    WHERE hypertable_name = 'container_stats'
                ) THEN
                    PERFORM drop_chunks('container_stats', INTERVAL '0 seconds');
                END IF;
            END $$;
            "#,
        )
        .await
        .ok();

        conn.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF EXISTS (
                    SELECT 1 FROM timescaledb_information.hypertables 
                    WHERE hypertable_name = 'container_logs'
                ) THEN
                    PERFORM drop_chunks('container_logs', INTERVAL '0 seconds');
                END IF;
            END $$;
            "#,
        )
        .await
        .ok();

        // Note: http_requests hypertable cleanup is handled in migration 000010 down method

        Ok(())
    }
}

