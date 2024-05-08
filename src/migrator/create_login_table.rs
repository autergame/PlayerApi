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
                    .table(Login::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Login::Id).integer().not_null().primary_key())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-login-id")
                            .from(Login::Table, Login::Id)
                            .to(Session::Table, Session::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Login::Server).string().not_null())
                    .col(ColumnDef::new(Login::Username).string().not_null())
                    .col(ColumnDef::new(Login::Password).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Login::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Login {
    Table,
    Id,
    Server,
    Username,
    Password,
}
