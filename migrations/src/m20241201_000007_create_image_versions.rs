use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ImageVersions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ImageVersions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ImageVersions::ImageId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ImageVersions::RepoTags)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ImageVersions::SizeBytes)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ImageVersions::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ImageVersions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_image_versions_image_id")
                    .table(ImageVersions::Table)
                    .col(ImageVersions::ImageId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_image_versions_timestamp")
                    .table(ImageVersions::Table)
                    .col(ImageVersions::Timestamp)
                    .to_owned(),
            )
            .await?;

        // Convert to hypertable
        let conn = manager.get_connection();
        conn.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM timescaledb_information.hypertables 
                    WHERE hypertable_name = 'image_versions'
                ) THEN
                    -- Drop the primary key constraint if it exists
                    IF EXISTS (
                        SELECT 1 FROM pg_constraint 
                        WHERE conname = 'image_versions_pkey'
                    ) THEN
                        ALTER TABLE image_versions DROP CONSTRAINT image_versions_pkey;
                    END IF;
                    
                    -- Convert to hypertable
                    PERFORM create_hypertable('image_versions', 'timestamp', 
                        chunk_time_interval => INTERVAL '1 day',
                        if_not_exists => TRUE);
                    
                    -- Recreate primary key as composite (id, timestamp) to satisfy TimescaleDB
                    ALTER TABLE image_versions ADD CONSTRAINT image_versions_pkey 
                        PRIMARY KEY (id, timestamp);
                END IF;
            END $$;
            "#,
        )
        .await
        .map_err(|e| DbErr::Custom(format!("Failed to convert image_versions to hypertable: {}", e)))?;

        log::info!("Created image_versions hypertable");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ImageVersions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ImageVersions {
    Table,
    Id,
    ImageId,
    RepoTags,
    SizeBytes,
    Timestamp,
    CreatedAt,
}

