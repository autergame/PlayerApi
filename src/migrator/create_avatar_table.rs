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
                    .table(Avatar::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Avatar::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Avatar::SessionId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-avatar-session_id")
                            .from(Avatar::Table, Avatar::SessionId)
                            .to(Session::Table, Session::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Avatar::Name).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Avatar::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Avatar {
    Table,
    Id,
    SessionId,
    Name,
}
