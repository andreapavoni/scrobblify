use anyhow::Result;

use super::{Album, Artist, Track, TrackInfo};

#[async_trait::async_trait]
pub trait Repository /* or `Repository: Send + Sync` */ {
    async fn insert_track(&self, track: Track) -> Result<()>;
    async fn insert_album(&self, album: Album) -> Result<()>;
    async fn insert_artist(&self, artist: Artist) -> Result<()>;
    async fn link_artist_track(&self, artist: Artist, track: Track) -> Result<()>;
    async fn link_album_track(&self, album: Album, track: Track) -> Result<()>;
    async fn link_album_artist(&self, album: Album, artist: Artist) -> Result<()>;
    async fn scrobble(&self, track_info: TrackInfo) -> Result<()>;
}
