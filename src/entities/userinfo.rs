use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::extra::opt_num_from_str_or_num;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "user_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i64,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub auth: Option<i64>,

    pub status: Option<String>,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub is_trial: Option<i64>,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub exp_date: Option<i64>,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub created_at: Option<i64>,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub active_cons: Option<i64>,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub max_connections: Option<i64>,
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
