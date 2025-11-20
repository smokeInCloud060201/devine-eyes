use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HttpRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HttpRequests::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(HttpRequests::ContainerId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HttpRequests::ContainerName)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HttpRequests::Endpoint)
                            .string_len(500)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HttpRequests::Method)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HttpRequests::HttpStatus)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HttpRequests::ResponseTimeMs)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HttpRequests::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HttpRequests::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_http_requests_container_id")
                    .table(HttpRequests::Table)
                    .col(HttpRequests::ContainerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_http_requests_timestamp")
                    .table(HttpRequests::Table)
                    .col(HttpRequests::Timestamp)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_http_requests_endpoint")
                    .table(HttpRequests::Table)
                    .col(HttpRequests::Endpoint)
                    .to_owned(),
            )
            .await?;

        // Convert to hypertable after creating the table
        let conn = manager.get_connection();
        conn.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM timescaledb_information.hypertables 
                    WHERE hypertable_name = 'http_requests'
                ) THEN
                    -- Drop the primary key constraint to convert to hypertable
                    IF EXISTS (
                        SELECT 1 FROM pg_constraint 
                        WHERE conname = 'http_requests_pkey'
                    ) THEN
                        ALTER TABLE http_requests DROP CONSTRAINT http_requests_pkey;
                    END IF;
                    
                    -- Convert to hypertable
                    PERFORM create_hypertable('http_requests', 'timestamp', 
                        chunk_time_interval => INTERVAL '1 day',
                        if_not_exists => TRUE);
                    
                    -- Recreate primary key as composite (id, timestamp) to satisfy TimescaleDB
                    ALTER TABLE http_requests ADD CONSTRAINT http_requests_pkey 
                        PRIMARY KEY (id, timestamp);
                END IF;
            END $$;
            "#,
        )
        .await
        .map_err(|e| DbErr::Custom(format!("Failed to convert http_requests to hypertable: {}", e)))?;

        // Add retention policy (keep raw data for 7 days)
        conn.execute_unprepared(
            r#"
            SELECT add_retention_policy('http_requests', 
                INTERVAL '7 days',
                if_not_exists => TRUE);
            "#,
        )
        .await
        .ok();

        // Enable compression (compress data older than 1 day)
        conn.execute_unprepared(
            r#"
            ALTER TABLE http_requests SET (
                timescaledb.compress,
                timescaledb.compress_segmentby = 'container_id'
            );
            SELECT add_compression_policy('http_requests', 
                INTERVAL '1 day',
                if_not_exists => TRUE);
            "#,
        )
        .await
        .ok();

        log::info!("Created http_requests table, converted to hypertable, and added retention/compression policies");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HttpRequests::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum HttpRequests {
    Table,
    Id,
    ContainerId,
    ContainerName,
    Endpoint,
    Method,
    HttpStatus,
    ResponseTimeMs,
    Timestamp,
    CreatedAt,
}

