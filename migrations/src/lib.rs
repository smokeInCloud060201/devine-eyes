use sea_orm_migration::prelude::*;

pub mod m20241201_000001_create_container_stats;
pub mod m20241201_000002_create_container_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241201_000001_create_container_stats::Migration),
            Box::new(m20241201_000002_create_container_logs::Migration),
        ]
    }
}

