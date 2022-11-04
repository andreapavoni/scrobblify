use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct HistoryPlayedTrack {
    pub track: TrackInfo,
    pub played_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default)]
pub struct CurrentPlayingTrack {
    pub track: Option<TrackInfo>,
    pub timestamp: Option<DateTime<Utc>>,
    pub progress_secs: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct Scrobble {
    pub timestamp: DateTime<Utc>,
    pub duration_secs: f64,
    pub track: TrackInfo,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub duration_secs: Duration,
    pub isrc: String,
}

#[derive(Clone, Debug)]
pub struct TrackInfo {
    pub id: String,
    pub title: String,
    pub album: Album,
    pub artists: Vec<Artist>,
    pub duration_secs: Duration,
    pub tags: Vec<String>,
    pub isrc: String,
    pub cover: String,
}

#[derive(Clone, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub cover: String,
}

#[derive(Clone, Debug)]
pub struct AlbumInfo {
    pub id: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub tracks: Vec<Track>,
    pub cover: String,
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
