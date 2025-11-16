use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DockerImages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DockerImages::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DockerImages::ImageId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DockerImages::RepoTags)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DockerImages::SizeBytes)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DockerImages::Architecture)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DockerImages::Os)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DockerImages::CreatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DockerImages::FirstSeen)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DockerImages::LastSeen)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index for image_id
        manager
            .create_index(
                Index::create()
                    .name("idx_docker_images_image_id")
                    .table(DockerImages::Table)
                    .col(DockerImages::ImageId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_docker_images_last_seen")
                    .table(DockerImages::Table)
                    .col(DockerImages::LastSeen)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DockerImages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DockerImages {
    Table,
    Id,
    ImageId,
    RepoTags,
    SizeBytes,
    Architecture,
    Os,
    CreatedAt,
    FirstSeen,
    LastSeen,
}

