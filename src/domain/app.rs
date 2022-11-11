use anyhow::Result;
use chrono::NaiveDate;

use super::models::*;

#[async_trait::async_trait]
pub trait App: Send + Sync {
    async fn scrobble(&self, scrobble: ScrobbleInfo) -> Result<()>;
    async fn get_recently_played(&self) -> Result<Vec<HistoryPlayedTrack>>;
    async fn get_currently_playing(&self) -> Result<Option<CurrentPlayingTrack>>;
    fn is_spotify_authenticated(&self) -> bool;
    async fn get_spotify_auth_url(&self) -> Result<String>;
    async fn store_spotify_auth_token(&self, code: &str) -> Result<()>;

    async fn stats_for_popular_tracks(&self, start: NaiveDate, end: NaiveDate) -> Vec<StatsTrack>;
    async fn stats_for_popular_tags(&self, start: NaiveDate, end: NaiveDate) -> Vec<StatsTag>;
    async fn stats_for_popular_artists(&self, start: NaiveDate, end: NaiveDate)
        -> Vec<StatsArtist>;
}
