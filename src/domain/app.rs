use anyhow::Result;

use super::models::*;

#[async_trait::async_trait]
pub trait App: Send + Sync {
    async fn scrobble(&self, scrobble: Scrobble) -> Result<()>;
    async fn scrobble_recently_played(&self) -> Result<()>;
    fn is_spotify_authenticated(&self) -> bool;
    async fn get_spotify_auth_url(&self) -> Result<String>;
    async fn store_spotify_auth_token(&self, code: &str) -> Result<()>;
}
