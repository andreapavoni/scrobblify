//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tags_tracks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub track_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tags::Entity",
        from = "Column::TagId",
        to = "super::tags::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Tags,
    #[sea_orm(
        belongs_to = "super::tracks::Entity",
        from = "Column::TrackId",
        to = "super::tracks::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Tracks,
}

impl Related<super::tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tags.def()
    }
}

impl Related<super::tracks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tracks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
