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
    pub scrobbled: bool,
}

impl PartialEq for CurrentPlayingTrack {
    fn eq(&self, other: &Self) -> bool {
        return self.timestamp == other.timestamp
            && self.track.clone().id == other.track.clone().id;
    }
}

#[derive(Clone, Debug)]
pub struct ScrobbleInfo {
    pub timestamp: DateTime<Utc>,
    pub duration_secs: f64,
    pub track: TrackInfo,
}

#[derive(Clone, Debug)]
pub struct Scrobble {
    pub timestamp: DateTime<Utc>,
    pub duration_secs: Duration,
    pub track: String,
    pub cover: String,
    pub album: String,
    pub artists: Vec<String>,
    pub tags: Vec<String>,
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
    pub tags: Vec<Tag>,
    pub isrc: String,
    pub cover: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    pub id: String,
}

#[derive(Clone, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub cover: String,
}

#[derive(Clone, Debug)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct StatsTag {
    pub name: String,
    pub score: u32,
    pub listened_secs: f64,
}

#[derive(Clone, Debug)]
pub struct StatsTrack {
    pub id: String,
    pub title: String,
    pub score: u32,
    pub listened_secs: f64,
}
#[derive(Clone, Debug)]
pub struct StatsArtist {
    pub id: String,
    pub name: String,
    pub score: u32,
    pub listened_secs: f64,
}
