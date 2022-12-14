use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Tracks table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tracks::Table)
                    .col(ColumnDef::new(Tracks::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Tracks::Title).string().not_null())
                    .col(ColumnDef::new(Tracks::DurationSecs).float().not_null())
                    .col(ColumnDef::new(Tracks::Isrc).not_null().string())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Tracks table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tracks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Tracks {
    Table,
    Id,
    Title,
    DurationSecs,
    Isrc,
}
