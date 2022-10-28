use sea_orm_migration::prelude::*;

use super::m20221022_000001_create_tracks_table::Tracks;
use super::m20221022_000003_create_albums_table::Albums;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221027_000003_create_albums_tracks_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the AlbumsTracks table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AlbumsTracks::Table)
                    .col(ColumnDef::new(AlbumsTracks::TrackId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-albums-tracks-track_id")
                            .from(AlbumsTracks::Table, AlbumsTracks::TrackId)
                            .to(Tracks::Table, Tracks::Id),
                    )
                    .col(ColumnDef::new(AlbumsTracks::AlbumId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-albums-tracks-album_id")
                            .from(AlbumsTracks::Table, AlbumsTracks::AlbumId)
                            .to(Albums::Table, Albums::Id),
                    )
                    .index(
                        Index::create()
                            .name("idx-albums-tracks-album_id-track_id")
                            .table(AlbumsTracks::Table)
                            .col(AlbumsTracks::AlbumId)
                            .col(AlbumsTracks::TrackId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the AlbumsTracks table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AlbumsTracks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AlbumsTracks {
    Table,
    AlbumId,
    TrackId,
}
