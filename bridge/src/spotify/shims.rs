use anyhow::Result;
use chrono::{DateTime, Utc};
use rspotify::model::{
    CurrentlyPlayingContext, FullTrack, PlayHistory, PlayableItem, SimplifiedAlbum,
    SimplifiedArtist,
};
use scrobblify_domain::{
    errors::SpotifyError,
    models::{
        Album as DomainAlbum, Artist as DomainArtist,
        CurrentPlayingTrack as DomainCurrentPlayingTrack,
        HistoryPlayedTrack as DomainHistoryPlayedTrack, Tag, TrackInfo as DomainTrackInfo,
    },
};
use std::time::Duration;

// FIXME: define new types in this crate, then (maybe) it's possible to build shims

#[derive(Clone, Debug)]
pub struct CurrentPlayingTrack {
    pub track: TrackInfo,
    pub timestamp: DateTime<Utc>,
    pub progress_secs: Duration,
    pub scrobbled: bool,
}

impl From<CurrentPlayingTrack> for DomainCurrentPlayingTrack {
    fn from(cpt: CurrentPlayingTrack) -> Self {
        Self {
            track: cpt.track.into(),
            timestamp: cpt.timestamp,
            progress_secs: cpt.progress_secs,
            scrobbled: cpt.scrobbled,
        }
    }
}

impl TryFrom<CurrentlyPlayingContext> for CurrentPlayingTrack {
    type Error = anyhow::Error;

    fn try_from(cpt: CurrentlyPlayingContext) -> Result<Self> {
        let full_track: FullTrack = match cpt.item {
            Some(PlayableItem::Track(ft)) => ft,
            _ => return Err(anyhow::Error::new(SpotifyError::TrackResponse)),
        };

        let progress_secs = cpt.progress.unwrap_or(Duration::new(0, 0));

        Ok(CurrentPlayingTrack {
            track: full_track.into(),
            timestamp: cpt.timestamp,
            progress_secs,
            scrobbled: false,
        })
    }
}

#[derive(Clone, Debug)]
pub struct HistoryPlayedTrack {
    pub track: TrackInfo,
    pub played_at: DateTime<Utc>,
}

impl From<HistoryPlayedTrack> for DomainHistoryPlayedTrack {
    fn from(hpt: HistoryPlayedTrack) -> Self {
        Self {
            track: hpt.track.into(),
            played_at: hpt.played_at,
        }
    }
}

impl TryFrom<PlayHistory> for HistoryPlayedTrack {
    type Error = anyhow::Error;

    fn try_from(ph: PlayHistory) -> Result<Self> {
        let track: TrackInfo = ph.track.try_into()?;

        Ok(Self {
            track,
            played_at: ph.played_at,
        })
    }
}

#[derive(Clone, Debug)]
pub struct TrackInfo {
    pub id: String,
    pub title: String,
    pub album: Album,
    pub artists: Vec<Artist>,
    pub duration_secs: Duration,
    pub tags: Vec<Tag>,
    pub isrc: String,
    pub cover: String,
}

impl From<TrackInfo> for DomainTrackInfo {
    fn from(t: TrackInfo) -> Self {
        Self {
            id: t.id,
            title: t.title,
            album: t.album.into(),
            artists: t.artists.into_iter().map(Into::into).collect(),
            duration_secs: t.duration_secs,
            tags: t.tags,
            isrc: t.isrc,
            cover: t.cover,
        }
    }
}

impl From<FullTrack> for TrackInfo {
    fn from(ft: FullTrack) -> Self {
        let artists: Vec<Artist> = ft.artists.into_iter().map(|a| a.into()).collect();
        let album: Album = ft.album.into();

        TrackInfo {
            id: ft.id.unwrap().as_ref().to_string(),
            title: ft.name,
            album: album.clone(),
            artists,
            duration_secs: ft.duration,
            tags: vec![],
            isrc: ft.external_ids.get("isrc").unwrap().to_string(),
            cover: album.cover,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub cover: String,
}

impl From<Album> for DomainAlbum {
    fn from(a: Album) -> Self {
        Self {
            id: a.id,
            title: a.title,
            cover: a.cover,
        }
    }
}

impl From<SimplifiedAlbum> for Album {
    fn from(sa: SimplifiedAlbum) -> Self {
        let cover = sa
            .images
            .into_iter()
            .find(|img| matches!(img.height, Some(640)))
            .map(|img| img.url)
            .unwrap_or_else(|| "".to_string());

        Album {
            id: sa.id.unwrap().as_ref().to_string(),
            title: sa.name,
            cover,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

impl From<Artist> for DomainArtist {
    fn from(a: Artist) -> Self {
        Self {
            id: a.id,
            name: a.name,
        }
    }
}

impl From<SimplifiedArtist> for Artist {
    fn from(sa: SimplifiedArtist) -> Self {
        Artist {
            id: sa.id.unwrap().as_ref().to_string(),
            name: sa.name,
        }
    }
}
