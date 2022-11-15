pub use sea_orm_migration;
use sea_orm_migration::prelude::*;

mod m20221022_000001_create_tracks_table;
mod m20221022_000002_create_scrobbles_table;
mod m20221022_000003_create_artists_table;
mod m20221022_000004_create_artists_tracks_table;
mod m20221027_000001_create_albums_table;
mod m20221027_000002_create_albums_artists_tracks_table;
mod m20221027_000003_create_albums_tracks_table;
mod m20221101_000001_create_tags_table;
mod m20221101_000002_create_tags_tracks_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20221022_000001_create_tracks_table::Migration),
            Box::new(m20221022_000002_create_scrobbles_table::Migration),
            Box::new(m20221022_000003_create_artists_table::Migration),
            Box::new(m20221022_000004_create_artists_tracks_table::Migration),
            Box::new(m20221027_000001_create_albums_table::Migration),
            Box::new(m20221027_000002_create_albums_artists_tracks_table::Migration),
            Box::new(m20221027_000003_create_albums_tracks_table::Migration),
            Box::new(m20221101_000001_create_tags_table::Migration),
            Box::new(m20221101_000002_create_tags_tracks_table::Migration),
        ]
    }
}
