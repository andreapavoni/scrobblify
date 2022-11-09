use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue, Database, DatabaseConnection, EntityTrait,
};
use std::{env, str::FromStr, time::Duration};

use crate::domain::{
    self,
    models::{Album, Artist, Track, TrackInfo},
};
use crate::{
    db::entities::{
        albums::{self, ActiveModel as AlbumsModel, Entity as AlbumEntity},
        albums_artists::{self, ActiveModel as AlbumsArtistsModel, Entity as AlbumsArtistsEntity},
        albums_tracks::{self, ActiveModel as AlbumsTracksModel, Entity as AlbumsTracksEntity},
        artists::{self, ActiveModel as ArtistsModel, Entity as ArtistEntity},
        artists_tracks::{self, ActiveModel as ArtistsTracksModel, Entity as ArtistsTracksEntity},
        scrobbles::{ActiveModel as ScrobblesModel, Entity as ScrobbleEntity},
        tags::{self, ActiveModel as TagsModel, Entity as TagEntity},
        tags_tracks::{self, ActiveModel as TagsTracksModel, Entity as TagsTracksEntity},
        tracks::{self, ActiveModel as TracksModel, Entity as TrackEntity},
    },
    domain::models::Tag,
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
impl domain::db::Repository for Repository {
    async fn insert_track(&self, track: Track) -> Result<()> {
        let new_track = TracksModel {
            id: ActiveValue::Set(track.id),
            title: ActiveValue::Set(track.title),
            duration_secs: ActiveValue::Set(track.duration_secs.as_secs_f64()),
            isrc: ActiveValue::Set(track.isrc),
        };

        TrackEntity::insert(new_track.clone())
            .on_conflict(
                OnConflict::column(tracks::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(to_db_error)?;

        Ok(())
    }

    async fn get_track_by_id(&self, id: String) -> Result<Option<Track>> {
        match TrackEntity::find_by_id(id).one(&self.conn).await? {
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

        AlbumEntity::insert(new_album.clone())
            .on_conflict(
                OnConflict::column(albums::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(to_db_error)?;

        Ok(())
    }

    async fn get_album_by_id(&self, id: String) -> Result<Option<Album>> {
        match AlbumEntity::find_by_id(id).one(&self.conn).await? {
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

        ArtistEntity::insert(new_artist.clone())
            .on_conflict(
                OnConflict::column(artists::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(to_db_error)?;
        Ok(())
    }

    async fn get_artist_by_id(&self, id: String) -> Result<Option<Artist>> {
        match ArtistEntity::find_by_id(id).one(&self.conn).await? {
            Some(artist) => Ok(Some(Artist {
                name: artist.name,
                id: artist.id,
            })),
            None => Ok(None),
        }
    }

    async fn insert_tag(&self, tag: Tag) -> Result<()> {
        let new_tag = TagsModel {
            id: ActiveValue::Set(tag.id),
        };

        TagEntity::insert(new_tag.clone())
            .on_conflict(OnConflict::column(tags::Column::Id).do_nothing().to_owned())
            .exec(&self.conn)
            .await
            .map_err(to_db_error)?;
        Ok(())
    }

    async fn get_tag_by_id(&self, id: String) -> Result<Option<Tag>> {
        match TagEntity::find_by_id(id).one(&self.conn).await? {
            Some(tag) => Ok(Some(Tag { id: tag.id })),
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

    async fn get_last_scrobble_timestamp(&self) -> Result<Option<DateTime<Utc>>> {
        match ScrobbleEntity::find().one(&self.conn).await? {
            Some(scrobble) => Ok(Some(DateTime::from_str(scrobble.timestamp.as_str())?)),
            None => Ok(None),
        }
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
    let tags = track_info.clone().tags;

    for tag in tags.iter() {
        let tags_tracks = TagsTracksModel {
            tag_id: ActiveValue::Set(tag.id.clone()),
            track_id: ActiveValue::Set(track.id.clone()),
        };

        TagsTracksEntity::insert(tags_tracks.clone())
            .on_conflict(
                OnConflict::columns(vec![
                    tags_tracks::Column::TagId,
                    tags_tracks::Column::TrackId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(conn)
            .await
            .map_err(to_db_error)?;
    }

    for artist in artists.iter() {
        let artists_tracks = ArtistsTracksModel {
            artist_id: ActiveValue::Set(artist.id.clone()),
            track_id: ActiveValue::Set(track.id.clone()),
        };

        ArtistsTracksEntity::insert(artists_tracks.clone())
            .on_conflict(
                OnConflict::columns(vec![
                    artists_tracks::Column::ArtistId,
                    artists_tracks::Column::TrackId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(conn)
            .await
            .map_err(to_db_error)?;

        let albums_artists = AlbumsArtistsModel {
            album_id: ActiveValue::Set(album.id.clone()),
            artist_id: ActiveValue::Set(artist.id.clone()),
        };

        AlbumsArtistsEntity::insert(albums_artists.clone())
            .on_conflict(
                OnConflict::columns(vec![
                    albums_artists::Column::ArtistId,
                    albums_artists::Column::AlbumId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(conn)
            .await
            .map_err(to_db_error)?;
    }

    let albums_tracks = AlbumsTracksModel {
        album_id: ActiveValue::Set(album.id),
        track_id: ActiveValue::Set(track.id),
    };

    AlbumsTracksEntity::insert(albums_tracks.clone())
        .on_conflict(
            OnConflict::columns(vec![
                albums_tracks::Column::TrackId,
                albums_tracks::Column::AlbumId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(conn)
        .await
        .map_err(to_db_error)?;

    Ok(())
}
