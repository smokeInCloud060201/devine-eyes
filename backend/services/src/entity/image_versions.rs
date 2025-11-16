use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "image_versions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub image_id: String,
    #[sea_orm(column_type = "Json")]
    pub repo_tags: Json,
    #[sea_orm(column_type = "BigInteger")]
    pub size_bytes: i64,
    pub timestamp: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

