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
                    .table(Watching::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Watching::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Watching::AvatarId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-watching-avatar_id")
                            .from(Watching::Table, Watching::AvatarId)
                            .to(Avatar::Table, Avatar::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Watching::Kind).string())
                    .col(ColumnDef::new(Watching::ValueId).integer())
                    .col(ColumnDef::new(Watching::Name).string())
                    .col(ColumnDef::new(Watching::Icon).string())
                    .col(ColumnDef::new(Watching::Time).integer().not_null())
                    .col(ColumnDef::new(Watching::EpisodeId).integer())
                    .col(ColumnDef::new(Watching::ContainerExtension).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Watching::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Watching {
    Table,
    Id,
    AvatarId,
    Kind,
    ValueId,
    Name,
    Icon,
    Time,
    EpisodeId,
    ContainerExtension,
}
