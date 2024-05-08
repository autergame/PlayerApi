use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "watching")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing)]
    pub id: i64,

    #[serde(skip_serializing)]
    pub avatar_id: i64,

    pub kind: Kind,

    #[serde(alias = "series_id", alias = "stream_id")]
    pub value_id: Option<i64>,

    pub name: Option<String>,

    #[serde(alias = "cover", alias = "stream_icon")]
    pub icon: Option<String>,

    pub time: i64,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<i64>,

    pub container_extension: Option<String>,
}

#[derive(Clone, Debug, PartialEq, EnumIter, Default, DeriveActiveEnum, Deserialize, Serialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
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
