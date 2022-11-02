use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, Database, DatabaseConnection, EntityTrait};
use std::time::Duration;

use crate::db::entities::{
    albums::ActiveModel as AlbumsModel,
    albums_artists::ActiveModel as AlbumsArtistsModel,
    albums_tracks::ActiveModel as AlbumsTracksModel,
    artists::ActiveModel as ArtistsModel,
    artists_tracks::ActiveModel as ArtistsTracksModel,
    scrobbles::ActiveModel as ScrobblesModel,
    tracks::{ActiveModel as TracksModel, Entity as TrackSchema},
};
use crate::domain::{
    self,
    models::{Album, Artist, Track, TrackInfo},
};

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

    // FIXME: remove allow(unused)
    #[allow(unused)]
    async fn link_artist_track(&self, artist: Artist, track: Track) -> Result<()> {
        let new_link = ArtistsTracksModel {
            artist_id: ActiveValue::Set(artist.id),
            track_id: ActiveValue::Set(track.id),
        };

        new_link.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }

    // FIXME: remove allow(unused)
    #[allow(unused)]
    async fn link_album_track(&self, album: Album, track: Track) -> Result<()> {
        let new_link = AlbumsTracksModel {
            album_id: ActiveValue::Set(album.id),
            track_id: ActiveValue::Set(track.id),
        };

        new_link.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }

    // FIXME: remove allow(unused)
    #[allow(unused)]
    async fn link_album_artist(&self, album: Album, artist: Artist) -> Result<()> {
        let new_link = AlbumsArtistsModel {
            album_id: ActiveValue::Set(album.id),
            artist_id: ActiveValue::Set(artist.id),
        };

        new_link.insert(&self.conn).await.map_err(to_db_error)?;
        Ok(())
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

    async fn get_track_by_id(&self, id: String) -> Result<Option<Track>> {
        match TrackSchema::find_by_id(id).one(&self.conn).await? {
            Some(track) => {
                println!("==> get track by id {:?}", track);
                Ok(Some(Track {
                    title: String::from("test"),
                    id: String::from("1234566"),
                    duration_secs: Duration::from_secs_f64(220.30),
                }))
            }
            None => Ok(None),
        }
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

    async fn insert_scrobble(&self, ti: TrackInfo) -> Result<()> {
        let scrobble = ScrobblesModel {
            timestamp: ActiveValue::Set(Utc::now().to_string()),
            origin: ActiveValue::Set(String::from("spotify")),
            duration_secs: ActiveValue::Set(ti.duration_secs.as_secs_f64()),
            track_id: ActiveValue::Set(ti.id),
        };

        scrobble.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }
}

/// Helper function to cast a sea_orm::DbErr into a domain Database Error.
/// This requires casting the sea_orm::DbErr into anyhow::Error first.
fn to_db_error(e: sea_orm::DbErr) -> domain::errors::DatabaseError {
    domain::errors::DatabaseError::from(anyhow::Error::from(e))
}
