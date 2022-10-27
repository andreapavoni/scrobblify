use anyhow::Result;
use rspotify::model::{CurrentlyPlayingContext, PlayableItem, SimplifiedAlbum, SimplifiedArtist};
use rspotify::model::{FullTrack, PlayHistory};

use crate::domain::{
    Album, Artist, CurrentPlayingTrack, HistoryPlayedTrack, SpotifyError, TrackInfo,
};

impl TryFrom<Option<CurrentlyPlayingContext>> for CurrentPlayingTrack {
    type Error = anyhow::Error;

    fn try_from(opt_cpt: Option<CurrentlyPlayingContext>) -> Result<Self> {
        let cpt = match opt_cpt {
            Some(cpt) => cpt,
            None => return Err(anyhow::Error::new(SpotifyError::TrackResponse)),
        };

        let full_track: FullTrack = match cpt.item {
            Some(pi) => match pi {
                PlayableItem::Track(ft) => ft,
                PlayableItem::Episode(_) => {
                    return Err(anyhow::Error::new(SpotifyError::TrackResponse))
                }
            },
            None => return Err(anyhow::Error::new(SpotifyError::TrackResponse)),
        };

        Ok(CurrentPlayingTrack {
            track: full_track.into(),
            timestamp: cpt.timestamp,
            progress_secs: cpt.progress.unwrap(),
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
            album,
            artists,
            duration_secs: ft.duration,
            genres: vec![],
        }
    }
}

impl From<SimplifiedAlbum> for Album {
    fn from(sa: SimplifiedAlbum) -> Self {
        Album {
            id: sa.id.unwrap().as_ref().to_string(),
            title: sa.name,
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