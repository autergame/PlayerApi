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
                    .table(Home::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Home::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Home::SessionId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-home-session_id")
                            .from(Home::Table, Home::SessionId)
                            .to(Session::Table, Session::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Home::Kind).string().not_null())
                    .col(ColumnDef::new(Home::ValueId).integer().not_null())
                    .col(ColumnDef::new(Home::Name).string().not_null())
                    .col(ColumnDef::new(Home::Icon).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Home::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Home {
    Table,
    Id,
    SessionId,
    Kind,
    ValueId,
    Name,
    Icon,
}
