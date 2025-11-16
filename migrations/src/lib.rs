use sea_orm_migration::prelude::*;

pub mod m20241201_000001_create_container_stats;
pub mod m20241201_000002_create_container_logs;
pub mod m20241201_000003_create_container_info;
pub mod m20241201_000004_enable_timescaledb;
pub mod m20241201_000005_convert_to_hypertables;
pub mod m20241201_000006_create_docker_images;
pub mod m20241201_000007_create_image_versions;
pub mod m20241201_000008_create_indexes_and_aggregates;
pub mod m20241201_000009_add_retention_policies;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241201_000001_create_container_stats::Migration),
            Box::new(m20241201_000002_create_container_logs::Migration),
            Box::new(m20241201_000003_create_container_info::Migration),
            Box::new(m20241201_000004_enable_timescaledb::Migration),
            Box::new(m20241201_000005_convert_to_hypertables::Migration),
            Box::new(m20241201_000006_create_docker_images::Migration),
            Box::new(m20241201_000007_create_image_versions::Migration),
            Box::new(m20241201_000008_create_indexes_and_aggregates::Migration),
            Box::new(m20241201_000009_add_retention_policies::Migration),
        ]
    }
}

