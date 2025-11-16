use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "docker_images")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub image_id: String,
    #[sea_orm(column_type = "Json")]
    pub repo_tags: Json,
    #[sea_orm(column_type = "BigInteger")]
    pub size_bytes: i64,
    pub architecture: Option<String>,
    pub os: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub first_seen: DateTimeWithTimeZone,
    pub last_seen: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

