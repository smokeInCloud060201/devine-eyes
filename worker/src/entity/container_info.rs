use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "container_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub container_id: String,
    pub container_name: String,
    pub image: String,
    pub status: String,
    pub created: Option<DateTimeWithTimeZone>,
    pub collected_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

