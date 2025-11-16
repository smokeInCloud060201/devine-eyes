use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ContainerInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContainerInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ContainerInfo::ContainerId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerInfo::ContainerName)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerInfo::Image)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerInfo::Status)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContainerInfo::Created)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ContainerInfo::CollectedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_container_info_container_id")
                    .table(ContainerInfo::Table)
                    .col(ContainerInfo::ContainerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_container_info_collected_at")
                    .table(ContainerInfo::Table)
                    .col(ContainerInfo::CollectedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ContainerInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ContainerInfo {
    Table,
    Id,
    ContainerId,
    ContainerName,
    Image,
    Status,
    Created,
    CollectedAt,
}

