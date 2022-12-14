use sea_orm_migration::prelude::*;

use super::m20221022_000003_create_artists_table::Artists;
use super::m20221027_000001_create_albums_table::Albums;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the AlbumsArtists table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AlbumsArtists::Table)
                    .col(ColumnDef::new(AlbumsArtists::AlbumId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-albums-artists-album_id")
                            .from(AlbumsArtists::Table, AlbumsArtists::AlbumId)
                            .to(Albums::Table, Albums::Id),
                    )
                    .col(ColumnDef::new(AlbumsArtists::ArtistId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-albums-artists-artist_id")
                            .from(AlbumsArtists::Table, AlbumsArtists::ArtistId)
                            .to(Artists::Table, Artists::Id),
                    )
                    .index(
                        Index::create()
                            .name("idx-albums-artists-album_id-artist_id")
                            .table(AlbumsArtists::Table)
                            .col(AlbumsArtists::AlbumId)
                            .col(AlbumsArtists::ArtistId)
                            .primary()
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the AlbumsArtists table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AlbumsArtists::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AlbumsArtists {
    Table,
    AlbumId,
    ArtistId,
}
