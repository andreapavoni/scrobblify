use sea_orm_migration::prelude::*;

use super::m20221022_000001_create_tracks_table::Tracks;
use super::m20221022_000003_create_artists_table::Artists;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the ArtistsTracks table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ArtistsTracks::Table)
                    .col(ColumnDef::new(ArtistsTracks::TrackId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-artists-tracks-track_id")
                            .from(ArtistsTracks::Table, ArtistsTracks::TrackId)
                            .to(Tracks::Table, Tracks::Id),
                    )
                    .col(ColumnDef::new(ArtistsTracks::ArtistId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-artists-tracks-artist_id")
                            .from(ArtistsTracks::Table, ArtistsTracks::ArtistId)
                            .to(Artists::Table, Artists::Id),
                    )
                    .index(
                        Index::create()
                            .name("idx-artists-tracks-artist_id-track_id")
                            .table(ArtistsTracks::Table)
                            .col(ArtistsTracks::ArtistId)
                            .col(ArtistsTracks::TrackId)
                            .primary()
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the ArtistsTracks table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ArtistsTracks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ArtistsTracks {
    Table,
    ArtistId,
    TrackId,
}
