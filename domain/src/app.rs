use anyhow::Result;

use super::{db::ParamsForStatsQuery, models::*};

#[async_trait::async_trait]
pub trait App: Send + Sync {
    fn get_current_track(&self) -> &Option<CurrentPlayingTrack>;
    fn set_current_track(&mut self, current_track: Option<CurrentPlayingTrack>);
    async fn scrobble(&self, scrobble: ScrobbleInfo) -> Result<()>;
    async fn get_recently_played(&self) -> Result<Vec<HistoryPlayedTrack>>;
    async fn get_currently_playing(&self) -> Result<Option<CurrentPlayingTrack>>;
    fn is_spotify_authenticated(&self) -> bool;
    async fn get_spotify_auth_url(&self) -> Result<String>;
    async fn store_spotify_auth_token(&self, code: &str) -> Result<()>;

    async fn stats_for_popular_tracks(&self, opts: ParamsForStatsQuery) -> Vec<StatsTrack>;
    async fn stats_for_popular_tags(&self, opts: ParamsForStatsQuery) -> Vec<StatsTag>;
    async fn stats_for_popular_artists(&self, opts: ParamsForStatsQuery) -> Vec<StatsArtist>;
}
