use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
//use utoipa::{IntoParams, ToSchema};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    DeriveEntityModel,
    /*ToSchema, IntoParams,*/ Deserialize,
    Serialize,
)]
#[sea_orm(table_name = "login")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i64,

    #[serde(skip_serializing)]
    //#[schema(example = "https://limetv.me", required = true)]
    pub server: String,
    //#[schema(example = "teste123", required = true)]
    pub username: String,
    //#[schema(example = "!@123098@!", required = true)]
    pub password: String,
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
