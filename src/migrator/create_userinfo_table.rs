use sea_orm_migration::prelude::*;

use super::create_session_table::Session;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserInfo::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_info-id")
                            .from(UserInfo::Table, UserInfo::Id)
                            .to(Session::Table, Session::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(UserInfo::Auth).integer().not_null())
                    .col(ColumnDef::new(UserInfo::Status).string().not_null())
                    .col(ColumnDef::new(UserInfo::IsTrial).integer().not_null())
                    .col(ColumnDef::new(UserInfo::ExpDate).integer().not_null())
                    .col(ColumnDef::new(UserInfo::CreatedAt).integer().not_null())
                    .col(ColumnDef::new(UserInfo::ActiveCons).integer().not_null())
                    .col(
                        ColumnDef::new(UserInfo::MaxConnections)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserInfo {
    Table,
    Id,
    Auth,
    Status,
    IsTrial,
    ExpDate,
    CreatedAt,
    ActiveCons,
    MaxConnections,
}
