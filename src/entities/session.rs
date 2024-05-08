use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "session")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing)]
    pub id: i64,

    pub auth_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::login::Entity")]
    Login,
    #[sea_orm(has_many = "super::userinfo::Entity")]
    UserInfo,
    #[sea_orm(has_many = "super::avatar::Entity")]
    Avatar,
}

impl Related<super::login::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Login.def()
    }
}

impl Related<super::userinfo::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserInfo.def()
    }
}

impl Related<super::avatar::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Avatar.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
