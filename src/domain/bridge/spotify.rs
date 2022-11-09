use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::domain::models::{CurrentPlayingTrack, HistoryPlayedTrack, Tag};
#[async_trait::async_trait]
pub trait SpotifyApi {
    fn has_auth(&self) -> bool;
    async fn get_auth_url(&self) -> Result<String>;
    async fn get_auth_token(&mut self, code: &str) -> Result<()>;
    async fn get_currently_playing(&self) -> Result<CurrentPlayingTrack>;
    async fn get_recently_played(
        &self,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<HistoryPlayedTrack>>;
    async fn get_tags(&self, artists_ids: Vec<&str>) -> Result<Vec<Tag>>;
}
