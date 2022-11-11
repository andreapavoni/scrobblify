use chrono::DateTime;
use std::str::FromStr;
use std::time::Duration;

use crate::{
    db::entities::{
        albums::Model as AlbumsModel, artists::Model as ArtistsModel, tags::Model as TagsModel,
        tracks::Model as TracksModel,
    },
    domain::models::{Album, Artist, Scrobble, Tag, Track},
};

use super::repository::ScrobbleQueryResult;

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

impl From<ScrobbleQueryResult> for Scrobble {
    fn from(s: ScrobbleQueryResult) -> Self {
        Self {
            timestamp: DateTime::from_str(s.timestamp.as_str()).unwrap(),
            duration_secs: Duration::from_secs_f64(s.duration_secs),
            track: s.track,
            cover: s.cover,
            album: s.album,
            artists: s
                .artists
                .as_str()
                .split(',')
                .into_iter()
                .map(|t| t.to_string())
                .collect(),
            tags: s
                .tags
                .as_str()
                .split(',')
                .into_iter()
                .map(|t| t.to_string())
                .collect(),
        }
    }
}
