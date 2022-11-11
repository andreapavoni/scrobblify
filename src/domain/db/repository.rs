use anyhow::Result;
use chrono::NaiveDate;

use crate::domain::models::{
    Album, Artist, Scrobble, StatsArtist, StatsTag, StatsTrack, Tag, Track, TrackInfo,
};

#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    // Tracks
    async fn insert_track(&self, track: Track) -> Result<()>;
    async fn get_track_by_id(&self, id: String) -> Result<Option<Track>>;

    // Albums
    async fn insert_album(&self, album: Album) -> Result<()>;

    // Artists
    async fn insert_artist(&self, artist: Artist) -> Result<()>;

    // Tags
    async fn insert_tag(&self, tag: Tag) -> Result<()>;

    // Scrobbles
    async fn insert_scrobble(&self, track_info: TrackInfo) -> Result<()>;
    async fn get_last_scrobble(&self) -> Result<Option<Scrobble>>;
    async fn list_scrobbles_by_date_range(&self, start: NaiveDate, end: NaiveDate)
        -> Vec<Scrobble>;
    async fn list_scrobbles_by_tag(&self, tag: &str) -> Vec<Scrobble>;
    async fn list_scrobbles_by_artist(&self, artist_id: &str) -> Vec<Scrobble>;

    // Stats
    async fn stats_for_popular_tags(&self, start: NaiveDate, end: NaiveDate) -> Vec<StatsTag>;
    async fn stats_for_popular_tracks(&self, start: NaiveDate, end: NaiveDate) -> Vec<StatsTrack>;
    async fn stats_for_popular_artists(&self, start: NaiveDate, end: NaiveDate)
        -> Vec<StatsArtist>;
}
