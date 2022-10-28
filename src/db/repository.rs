use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, Database, DatabaseConnection};

use crate::db::entities::albums::ActiveModel as AlbumsModel;
use crate::db::entities::albums_artists::ActiveModel as AlbumsArtistsModel;
use crate::db::entities::albums_tracks::ActiveModel as AlbumsTracksModel;
use crate::db::entities::artists::ActiveModel as ArtistsModel;
use crate::db::entities::artists_tracks::ActiveModel as ArtistsTracksModel;
use crate::db::entities::scrobbles::ActiveModel as ScrobblesModel;
use crate::db::entities::tracks::ActiveModel as TracksModel;

use crate::domain::{self, Album, Artist, Track, TrackInfo};

#[derive(Clone)]
pub struct Repository {
    conn: DatabaseConnection,
}

impl Repository {
    pub async fn new(url: String) -> Result<Repository> {
        let conn = Database::connect(url).await?;
        Ok(Repository { conn })
    }

    pub fn with_connection(conn: DatabaseConnection) -> Repository {
        Repository { conn }
    }

    pub fn conn(&self) -> DatabaseConnection {
        self.conn.clone()
    }
}

#[async_trait::async_trait]
impl domain::repository::Repository for Repository {
    async fn insert_track(&self, track: Track) -> Result<()> {
        let new_track = TracksModel {
            id: ActiveValue::Set(track.id),
            title: ActiveValue::Set(track.title),
            duration_secs: ActiveValue::Set(track.duration_secs.as_secs_f64()),
        };

        new_track.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }

    async fn insert_album(&self, album: Album) -> Result<()> {
        let new_album = AlbumsModel {
            id: ActiveValue::Set(album.id),
            title: ActiveValue::Set(album.title),
        };

        new_album.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }

    async fn insert_artist(&self, artist: Artist) -> Result<()> {
        let new_artist = ArtistsModel {
            id: ActiveValue::Set(artist.id),
            name: ActiveValue::Set(artist.name),
        };

        new_artist.insert(&self.conn).await.map_err(to_db_error)?;
        Ok(())
    }

    async fn link_artist_track(&self, artist: Artist, track: Track) -> Result<()> {
        let new_link = ArtistsTracksModel {
            artist_id: ActiveValue::Set(artist.id),
            track_id: ActiveValue::Set(track.id),
        };

        new_link.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }

    async fn link_album_track(&self, album: Album, track: Track) -> Result<()> {
        let new_link = AlbumsTracksModel {
            album_id: ActiveValue::Set(album.id),
            track_id: ActiveValue::Set(track.id),
        };

        new_link.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }

    async fn link_album_artist(&self, album: Album, artist: Artist) -> Result<()> {
        let new_link = AlbumsArtistsModel {
            album_id: ActiveValue::Set(album.id),
            artist_id: ActiveValue::Set(artist.id),
        };

        new_link.insert(&self.conn).await.map_err(to_db_error)?;
        Ok(())
    }

    async fn scrobble(&self, ti: TrackInfo) -> Result<()> {
        let new_link = ScrobblesModel {
            timestamp: ActiveValue::Set(Utc::now().to_string()),
            origin: ActiveValue::Set(String::from("spotify")),
            duration_secs: ActiveValue::Set(ti.duration_secs.as_secs_f64()),
            track_id: ActiveValue::Set(ti.id),
        };

        new_link.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }
}

/// Helper function to cast a sea_orm::DbErr into a domain Database Error.
/// This requires casting the sea_orm::DbErr into anyhow::Error first.
pub fn to_db_error(e: sea_orm::DbErr) -> domain::DatabaseError {
    domain::DatabaseError::from(anyhow::Error::from(e))
}
