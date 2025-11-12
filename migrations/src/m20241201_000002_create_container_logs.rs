use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ContainerLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContainerLogs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ContainerLogs::ContainerId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerLogs::ContainerName)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ContainerLogs::LogLine).text().not_null())
                    .col(
                        ColumnDef::new(ContainerLogs::Stream)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerLogs::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerLogs::CreatedAt)
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
                    .name("idx_container_logs_container_id")
                    .table(ContainerLogs::Table)
                    .col(ContainerLogs::ContainerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_container_logs_timestamp")
                    .table(ContainerLogs::Table)
                    .col(ContainerLogs::Timestamp)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ContainerLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ContainerLogs {
    Table,
    Id,
    ContainerId,
    ContainerName,
    LogLine,
    Stream,
    Timestamp,
    CreatedAt,
}

