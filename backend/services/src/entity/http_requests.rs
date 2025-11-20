use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "http_requests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub container_id: String,
    pub container_name: String,
    pub endpoint: String,
    pub method: String,
    #[sea_orm(column_type = "SmallInteger")]
    pub http_status: i16,
    #[sea_orm(column_type = "Double")]
    pub response_time_ms: f64,
    pub timestamp: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

