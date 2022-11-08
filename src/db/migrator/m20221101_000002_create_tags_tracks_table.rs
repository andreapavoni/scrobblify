use sea_orm_migration::prelude::*;

use super::m20221022_000001_create_tracks_table::Tracks;
use super::m20221101_000001_create_tags_table::Tags;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the TagsTracks table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TagsTracks::Table)
                    .col(ColumnDef::new(TagsTracks::TrackId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tags-tracks-track_id")
                            .from(TagsTracks::Table, TagsTracks::TrackId)
                            .to(Tracks::Table, Tracks::Id),
                    )
                    .col(ColumnDef::new(TagsTracks::TagId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tags-tracks-tag_id")
                            .from(TagsTracks::Table, TagsTracks::TagId)
                            .to(Tags::Table, Tags::Id),
                    )
                    .index(
                        Index::create()
                            .name("idx-tags-tracks-tag_id-track_id")
                            .table(TagsTracks::Table)
                            .col(TagsTracks::TagId)
                            .col(TagsTracks::TrackId)
                            .primary()
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the TagsTracks table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TagsTracks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TagsTracks {
    Table,
    TagId,
    TrackId,
}
