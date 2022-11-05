//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "artists_tracks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub track_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub artist_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::artists::Entity",
        from = "Column::ArtistId",
        to = "super::artists::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Artists,
    #[sea_orm(
        belongs_to = "super::tracks::Entity",
        from = "Column::TrackId",
        to = "super::tracks::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Tracks,
}

impl Related<super::artists::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Artists.def()
    }
}

impl Related<super::tracks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tracks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
