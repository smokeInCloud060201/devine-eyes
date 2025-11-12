use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ContainerStats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContainerStats::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::ContainerId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::ContainerName)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::CpuUsagePercent)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::MemoryUsageBytes)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::MemoryLimitBytes)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::MemoryUsagePercent)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::NetworkRxBytes)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::NetworkTxBytes)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::BlockReadBytes)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::BlockWriteBytes)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerStats::CreatedAt)
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
                    .name("idx_container_stats_container_id")
                    .table(ContainerStats::Table)
                    .col(ContainerStats::ContainerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_container_stats_timestamp")
                    .table(ContainerStats::Table)
                    .col(ContainerStats::Timestamp)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ContainerStats::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ContainerStats {
    Table,
    Id,
    ContainerId,
    ContainerName,
    CpuUsagePercent,
    MemoryUsageBytes,
    MemoryLimitBytes,
    MemoryUsagePercent,
    NetworkRxBytes,
    NetworkTxBytes,
    BlockReadBytes,
    BlockWriteBytes,
    Timestamp,
    CreatedAt,
}

