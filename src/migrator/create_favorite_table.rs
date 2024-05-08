use sea_orm_migration::prelude::*;

use super::create_avatar_table::Avatar;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Favorite::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Favorite::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Favorite::AvatarId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-favorite-avatar_id")
                            .from(Favorite::Table, Favorite::AvatarId)
                            .to(Avatar::Table, Avatar::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Favorite::Kind).string())
                    .col(ColumnDef::new(Favorite::ValueId).integer())
                    .col(ColumnDef::new(Favorite::Name).string())
                    .col(ColumnDef::new(Favorite::Icon).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Favorite::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Favorite {
    Table,
    Id,
    AvatarId,
    Kind,
    ValueId,
    Name,
    Icon,
}
