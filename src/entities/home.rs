use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Default, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "home")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing)]
    pub id: i64,

    #[serde(skip_serializing)]
    pub session_id: i64,

    #[serde(skip_serializing_if = "Kind::is_normal")]
    pub kind: Kind,

    pub value_id: i64,
    pub name: String,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Default, EnumIter, DeriveActiveEnum, Serialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
    #[serde(rename = "movie")]
    #[sea_orm(string_value = "TopMovie")]
    TopMovie,

    #[serde(rename = "serie")]
    #[sea_orm(string_value = "TopSerie")]
    TopSerie,

    #[sea_orm(string_value = "Movie")]
    Movie,

    #[sea_orm(string_value = "Serie")]
    Serie,
}

impl Kind {
    pub fn is_normal(&self) -> bool {
        match self {
            Kind::TopMovie => false,
            Kind::TopSerie => false,
            Kind::Movie => true,
            Kind::Serie => true,
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::session::Entity",
        from = "Column::SessionId",
        to = "super::session::Column::Id"
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
