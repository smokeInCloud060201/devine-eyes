use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "container_stats")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub container_id: String,
    pub container_name: String,
    #[sea_orm(column_type = "Double")]
    pub cpu_usage_percent: f64,
    #[sea_orm(column_type = "BigInteger")]
    pub memory_usage_bytes: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub memory_limit_bytes: i64,
    #[sea_orm(column_type = "Double")]
    pub memory_usage_percent: f64,
    #[sea_orm(column_type = "BigInteger")]
    pub network_rx_bytes: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub network_tx_bytes: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub block_read_bytes: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub block_write_bytes: i64,
    pub timestamp: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

