use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, Database, DatabaseConnection, EntityTrait};
use std::{env, time::Duration};

use crate::db::entities::{
    albums::{ActiveModel as AlbumsModel, Entity as AlbumSchema},
    albums_artists::ActiveModel as AlbumsArtistsModel,
    albums_tracks::ActiveModel as AlbumsTracksModel,
    artists::{ActiveModel as ArtistsModel, Entity as ArtistSchema},
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
    pub async fn new_from_env() -> Result<Repository> {
        let url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let conn = Database::connect(url).await?;
        Ok(Repository { conn })
    }

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
            isrc: ActiveValue::Set(track.isrc),
        };

        new_track.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }

    async fn get_track_by_id(&self, id: String) -> Result<Option<Track>> {
        match TrackSchema::find_by_id(id).one(&self.conn).await? {
            Some(track) => Ok(Some(Track {
                title: track.title,
                id: track.id,
                duration_secs: Duration::from_secs_f64(track.duration_secs),
                isrc: track.isrc,
            })),
            None => Ok(None),
        }
    }

    async fn insert_album(&self, album: Album) -> Result<()> {
        let new_album = AlbumsModel {
            id: ActiveValue::Set(album.id),
            title: ActiveValue::Set(album.title),
            cover: ActiveValue::Set(album.cover),
        };

        new_album.insert(&self.conn).await.map_err(to_db_error)?;

        Ok(())
    }

    async fn get_album_by_id(&self, id: String) -> Result<Option<Album>> {
        match AlbumSchema::find_by_id(id).one(&self.conn).await? {
            Some(album) => Ok(Some(Album {
                title: album.title,
                id: album.id,
                cover: album.cover,
            })),
            None => Ok(None),
        }
    }

    async fn insert_artist(&self, artist: Artist) -> Result<()> {
        let new_artist = ArtistsModel {
            id: ActiveValue::Set(artist.id),
            name: ActiveValue::Set(artist.name),
        };

        new_artist.insert(&self.conn).await.map_err(to_db_error)?;
        Ok(())
    }

    async fn get_artist_by_id(&self, id: String) -> Result<Option<Artist>> {
        match ArtistSchema::find_by_id(id).one(&self.conn).await? {
            Some(artist) => Ok(Some(Artist {
                name: artist.name,
                id: artist.id,
            })),
            None => Ok(None),
        }
    }

    async fn insert_scrobble(&self, track_info: TrackInfo) -> Result<()> {
        let scrobble = ScrobblesModel {
            timestamp: ActiveValue::Set(Utc::now().to_string()),
            origin: ActiveValue::Set(String::from("spotify")),
            duration_secs: ActiveValue::Set(track_info.duration_secs.as_secs_f64()),
            track_id: ActiveValue::Set(track_info.clone().id),
        };

        scrobble.insert(&self.conn).await.map_err(to_db_error)?;

        insert_entity_links(&self.conn, track_info.clone()).await?;

        Ok(())
    }
}

/// Helper function to cast a sea_orm::DbErr into a domain Database Error.
/// This requires casting the sea_orm::DbErr into anyhow::Error first.
fn to_db_error(e: sea_orm::DbErr) -> domain::errors::DatabaseError {
    domain::errors::DatabaseError::from(anyhow::Error::from(e))
}

async fn insert_entity_links(conn: &DatabaseConnection, track_info: TrackInfo) -> Result<()> {
    let track: Track = track_info.clone().into();
    let artists = track_info.clone().artists;
    let album = track_info.clone().album;

    for artist in artists.iter() {
        let artists_tracks = ArtistsTracksModel {
            artist_id: ActiveValue::Set(artist.id.clone()),
            track_id: ActiveValue::Set(track.id.clone()),
        };

        let _ = artists_tracks.insert(conn).await.map_err(to_db_error);

        let albums_artists = AlbumsArtistsModel {
            album_id: ActiveValue::Set(album.id.clone()),
            artist_id: ActiveValue::Set(artist.id.clone()),
        };

        let _ = albums_artists.insert(conn).await.map_err(to_db_error);
    }

    let albums_tracks = AlbumsTracksModel {
        album_id: ActiveValue::Set(album.id),
        track_id: ActiveValue::Set(track.id),
    };

    let _ = albums_tracks.insert(conn).await.map_err(to_db_error);

    Ok(())
}
