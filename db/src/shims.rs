use scrobblify_domain::models::{Album, Artist, Tag, Track};
use std::time::Duration;

use crate::entities::{
    albums::Model as AlbumsModel, artists::Model as ArtistsModel, tags::Model as TagsModel,
    tracks::Model as TracksModel,
};

impl From<TagsModel> for Tag {
    fn from(t: TagsModel) -> Self {
        Self {
            id: t.id.to_owned(),
        }
    }
}

impl From<TracksModel> for Track {
    fn from(t: TracksModel) -> Self {
        Self {
            title: t.title,
            id: t.id,
            duration_secs: Duration::from_secs_f64(t.duration_secs),
            isrc: t.isrc,
        }
    }
}

impl From<AlbumsModel> for Album {
    fn from(a: AlbumsModel) -> Self {
        Self {
            title: a.title,
            id: a.id,
            cover: a.cover,
        }
    }
}

impl From<ArtistsModel> for Artist {
    fn from(a: ArtistsModel) -> Self {
        Self {
            name: a.name,
            id: a.id,
        }
    }
}
