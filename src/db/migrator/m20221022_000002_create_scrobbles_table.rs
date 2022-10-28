use sea_orm_migration::prelude::*;

use super::m20221022_000001_create_tracks_table::Tracks;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221022_000002_create_scrobbles_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Scrobbles table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Scrobbles::Table)
                    .col(
                        ColumnDef::new(Scrobbles::Timestamp)
                            .timestamp()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Scrobbles::Origin).string().not_null())
                    .col(ColumnDef::new(Scrobbles::DurationSecs).float().not_null())
                    .col(ColumnDef::new(Scrobbles::TrackId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-scrobbles-track_id")
                            .from(Scrobbles::Table, Scrobbles::TrackId)
                            .to(Tracks::Table, Tracks::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Scrobbles table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Scrobbles::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Scrobbles {
    Table,
    Timestamp,
    Origin,
    DurationSecs,
    TrackId,
}
