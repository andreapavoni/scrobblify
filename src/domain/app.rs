use anyhow::Result;

use super::models::*;

#[async_trait::async_trait]
pub trait App {
    // All the infos about Track, Album, Artist, Tags and Scrobble itself will be inserted on db
    async fn scrobble(&self, scrobble: Scrobble) -> Result<()>;
}
