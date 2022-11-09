use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::domain::models::{Album, Artist, Tag, Track, TrackInfo};

#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    // Tracks
    async fn insert_track(&self, track: Track) -> Result<()>;
    async fn get_track_by_id(&self, id: String) -> Result<Option<Track>>;
    // TODO: async fn list_tracks(&self, Option<PaginationOptions or ListQueryOptions>) -> Vec<Track>;
    // TODO: async fn get_track_info_by_id(&self, id: String) -> Result<Option<TrackInfo>>;
    // TODO: async fn list_tracks_infos(&self, Option<PaginationOptions or ListQueryOptions>) -> Vec<TrackInfo>;

    // Albums
    async fn insert_album(&self, album: Album) -> Result<()>;
    async fn get_album_by_id(&self, id: String) -> Result<Option<Album>>;
    // TODO: async fn list_albums(&self, Option<PaginationOptions or ListQueryOptions>) -> Vec<Album>;
    // TODO: async fn get_album_info_by_id(&self, id: String) -> Result<Option<AlbumInfo>>;
    // TODO: async fn list_albums_infos(&self, Option<PaginationOptions or ListQueryOptions>) -> Vec<AlbumInfo>;

    // Artists
    async fn insert_artist(&self, artist: Artist) -> Result<()>;
    async fn get_artist_by_id(&self, id: String) -> Result<Option<Artist>>;
    // TODO: async fn list_artists(&self, Option<PaginationOptions or ListQueryOptions>) -> Vec<Artist>;
    // TODO: async fn get_artist_info_by_id(&self, id: String) -> Result<Option<ArtistInfo>>;
    // TODO: async fn list_artists_infos(&self, Option<PaginationOptions or ListQueryOptions>) -> Vec<ArtistInfo>;

    // Tags
    async fn insert_tag(&self, tag: Tag) -> Result<()>;
    async fn get_tag_by_id(&self, id: String) -> Result<Option<Tag>>;

    // Scrobbles
    async fn insert_scrobble(&self, track_info: TrackInfo) -> Result<()>;
    async fn get_last_scrobble_timestamp(&self) -> Result<Option<DateTime<Utc>>>;
    // TODO: async fn list_scrobbles(&self, Option<PaginationOptions or ListQueryOptions>) -> Vec<Scrobble>;
}