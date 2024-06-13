use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::extra::{default_on_null, num_from_str_or_num};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "user_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i64,

    #[serde(default)]
    #[serde(deserialize_with = "num_from_str_or_num")]
    pub auth: i64,

    #[serde(default)]
    #[serde(deserialize_with = "default_on_null")]
    pub status: String,

    #[serde(default)]
    #[serde(deserialize_with = "num_from_str_or_num")]
    pub is_trial: i64,

    #[serde(default)]
    #[serde(deserialize_with = "num_from_str_or_num")]
    pub exp_date: i64,

    #[serde(default)]
    #[serde(deserialize_with = "num_from_str_or_num")]
    pub created_at: i64,

    #[serde(default)]
    #[serde(deserialize_with = "num_from_str_or_num")]
    pub active_cons: i64,

    #[serde(default)]
    #[serde(deserialize_with = "num_from_str_or_num")]
    pub max_connections: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::session::Entity",
        from = "Column::Id",
        to = "super::session::Column::Id",
        on_delete = "Cascade"
    )]
    Session,
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
