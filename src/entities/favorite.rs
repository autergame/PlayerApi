use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "favorite")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing)]
    pub id: i64,

    #[serde(skip_serializing)]
    pub avatar_id: i64,

    #[serde(skip_serializing)]
    pub kind: Kind,

    pub value_id: i64,
    pub name: String,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Default, EnumIter, DeriveActiveEnum, Deserialize, Serialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
    #[sea_orm(string_value = "Live")]
    Live,

    #[sea_orm(string_value = "Movie")]
    Movie,

    #[sea_orm(string_value = "Serie")]
    Serie,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::avatar::Entity",
        from = "Column::AvatarId",
        to = "super::avatar::Column::Id"
		on_delete = "Cascade"
    )]
    Avatar,
}

impl Related<super::avatar::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Avatar.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
