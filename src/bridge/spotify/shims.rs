use anyhow::Result;
use rspotify::model::{CurrentlyPlayingContext, PlayableItem, SimplifiedAlbum, SimplifiedArtist};
use rspotify::model::{FullTrack, PlayHistory};
use std::time::Duration;

use crate::domain::models::Track;
use crate::domain::{
    errors::SpotifyError,
    models::{Album, Artist, CurrentPlayingTrack, HistoryPlayedTrack, TrackInfo},
};

impl TryFrom<CurrentlyPlayingContext> for CurrentPlayingTrack {
    type Error = anyhow::Error;

    fn try_from(cpt: CurrentlyPlayingContext) -> Result<Self> {
        let full_track: FullTrack = match cpt.item {
            Some(PlayableItem::Track(ft)) => ft,
            _ => return Err(anyhow::Error::new(SpotifyError::TrackResponse)),
        };

        let progress_secs = match cpt.progress {
            Some(progress) => progress,
            None => Duration::new(0, 0),
        };

        Ok(CurrentPlayingTrack {
            track: full_track.into(),
            timestamp: cpt.timestamp,
            progress_secs,
            scrobbled: false,
        })
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

impl From<TrackInfo> for Track {
    fn from(track_info: TrackInfo) -> Self {
        Track {
            id: track_info.id,
            title: track_info.title,
            duration_secs: track_info.duration_secs,
            isrc: track_info.isrc,
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

impl From<SimplifiedArtist> for Artist {
    fn from(sa: SimplifiedArtist) -> Self {
        Artist {
            id: sa.id.unwrap().as_ref().to_string(),
            name: sa.name,
        }
    }
}
