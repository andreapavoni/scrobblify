use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct HistoryPlayedTrack {
    pub track: TrackInfo,
    pub played_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CurrentPlayingTrack {
    pub track: TrackInfo,
    pub timestamp: DateTime<Utc>,
    pub progress_secs: Duration,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub duration_secs: Duration,
}

#[derive(Clone, Debug)]
pub struct TrackInfo {
    pub id: String,
    pub title: String,
    pub album: Album,
    pub artists: Vec<Artist>,
    pub duration_secs: Duration,
    pub genres: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct AlbumInfo {
    pub id: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct ArtistInfo {
    pub id: String,
    pub name: String,
    pub albums: Vec<Album>,
    pub tracks: Vec<TrackInfo>,
}
