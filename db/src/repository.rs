use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue, Database, DatabaseConnection, DbBackend,
    EntityTrait, FromQueryResult, Statement,
};
use std::{env, str::FromStr, time::Duration};

use scrobblify_domain::{
    self,
    db::ParamsForStatsQuery,
    models::{Album, Artist, Scrobble, StatsArtist, StatsTag, StatsTrack, Tag, Track, TrackInfo},
};

use crate::entities::{
    albums::{self, ActiveModel as AlbumsModel, Entity as AlbumEntity},
    albums_artists::{self, ActiveModel as AlbumsArtistsModel, Entity as AlbumsArtistsEntity},
    albums_tracks::{self, ActiveModel as AlbumsTracksModel, Entity as AlbumsTracksEntity},
    artists::{self, ActiveModel as ArtistsModel, Entity as ArtistEntity},
    artists_tracks::{self, ActiveModel as ArtistsTracksModel, Entity as ArtistsTracksEntity},
    scrobbles::ActiveModel as ScrobblesModel,
    tags::{self, ActiveModel as TagsModel, Entity as TagEntity},
    tags_tracks::{self, ActiveModel as TagsTracksModel, Entity as TagsTracksEntity},
    tracks::{self, ActiveModel as TracksModel, Entity as TrackEntity},
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

#[derive(Debug, FromQueryResult)]
struct ScrobbleQueryResult {
    track: String,
    duration_secs: f64,
    cover: String,
    artists: String,
    album: String,
    tags: String,
    timestamp: String,
}

#[derive(Debug, FromQueryResult)]
struct PopularTagQueryResult {
    tag: String,
    score: u32,
    listened_secs: f64,
}

#[derive(Debug, FromQueryResult)]
struct PopularTrackQueryResult {
    id: String,
    title: String,
    score: u32,
    listened_secs: f64,
}

#[derive(Debug, FromQueryResult)]
struct PopularArtistQueryResult {
    id: String,
    name: String,
    score: u32,
    listened_secs: f64,
}

#[async_trait::async_trait]
impl scrobblify_domain::db::Repository for Repository {
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
            Some(track) => Ok(Some(track.into())),
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

    async fn get_last_scrobble(&self) -> Result<Option<Scrobble>> {
        // match ScrobbleEntity::find()
        //     .join(JoinType::LeftJoin, scrobbles::Relation::Tracks.def())
        //     .into_model::<ScrobbleQueryResult>()
        //     .one(&self.conn)
        //     .await?
        // {
        //     Some(scrobble) => Ok(Some(scrobble.into())),
        //     None => Ok(None),
        // }
        match ScrobbleQueryResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            include_str!("queries/get_last_scrobble_query.sql"),
            vec![1.into()],
        ))
        .one(&self.conn)
        .await?
        {
            Some(scrobble) => Ok(Some(scrobble.into())),
            None => Ok(None),
        }
    }

    async fn list_scrobbles_by_date_range(&self, opts: ParamsForStatsQuery) -> Vec<Scrobble> {
        let (start, end) = build_dates_range(opts);

        match ScrobbleQueryResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            include_str!("queries/list_scrobbles_by_date_range_query.sql"),
            vec![
                sea_orm::Value::from(start.to_string()),
                sea_orm::Value::from(end.to_string()),
            ],
        ))
        .all(&self.conn)
        .await
        {
            Ok(scrobbles) => scrobbles.into_iter().map(|s| s.into()).collect(),
            Err(_) => vec![],
        }
    }

    async fn list_scrobbles_by_tag(&self, tag: &str) -> Vec<Scrobble> {
        match ScrobbleQueryResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            include_str!("queries/list_scrobbles_by_tag_query.sql"),
            vec![tag.into()],
        ))
        .all(&self.conn)
        .await
        {
            Ok(scrobbles) => scrobbles.into_iter().map(|s| s.into()).collect(),
            Err(_) => vec![],
        }
    }

    async fn list_scrobbles_by_artist(&self, artist_id: &str) -> Vec<Scrobble> {
        match ScrobbleQueryResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            include_str!("queries/list_scrobbles_by_artist_query.sql"),
            vec![artist_id.into()],
        ))
        .all(&self.conn)
        .await
        {
            Ok(scrobbles) => scrobbles.into_iter().map(|s| s.into()).collect(),
            Err(err) => {
                tracing::error!(
                    msg = "list scrobbles by artist query error",
                    error = format!("{:?}", err)
                );
                vec![]
            }
        }
    }

    async fn stats_for_popular_tags(&self, opts: ParamsForStatsQuery) -> Vec<StatsTag> {
        let (start, end) = build_dates_range(opts.clone());
        let limit = opts.limit.unwrap_or(10);

        match PopularTagQueryResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            include_str!("queries/stats_for_popular_tags.sql"),
            vec![
                sea_orm::Value::from(start.to_string()),
                sea_orm::Value::from(end.to_string()),
                sea_orm::Value::from(limit),
            ],
        ))
        .all(&self.conn)
        .await
        {
            Ok(tracks) => tracks.into_iter().map(|t| t.into()).collect(),
            Err(err) => {
                tracing::error!(
                    msg = "popular tags query error",
                    error = format!("{:?}", err)
                );
                vec![]
            }
        }
    }

    async fn stats_for_popular_tracks(&self, opts: ParamsForStatsQuery) -> Vec<StatsTrack> {
        let (start, end) = build_dates_range(opts.clone());
        let limit = opts.limit.unwrap_or(10);

        match PopularTrackQueryResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            include_str!("queries/stats_for_popular_tracks.sql"),
            vec![
                sea_orm::Value::from(start.to_string()),
                sea_orm::Value::from(end.to_string()),
                sea_orm::Value::from(limit),
            ],
        ))
        .all(&self.conn)
        .await
        {
            Ok(tracks) => tracks.into_iter().map(|t| t.into()).collect(),
            Err(err) => {
                tracing::error!(
                    msg = "popular tracks query error",
                    error = format!("{:?}", err)
                );
                vec![]
            }
        }
    }

    async fn stats_for_popular_artists(&self, opts: ParamsForStatsQuery) -> Vec<StatsArtist> {
        let (start, end) = build_dates_range(opts.clone());
        let limit = opts.limit.unwrap_or(10);

        match PopularArtistQueryResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            include_str!("queries/stats_for_popular_artists.sql"),
            vec![
                sea_orm::Value::from(start.to_string()),
                sea_orm::Value::from(end.to_string()),
                sea_orm::Value::from(limit),
            ],
        ))
        .all(&self.conn)
        .await
        {
            Ok(tracks) => tracks.into_iter().map(|t| t.into()).collect(),
            Err(err) => {
                tracing::error!(
                    msg = "popular artists query error",
                    error = format!("{:?}", err)
                );
                vec![]
            }
        }
    }
}

/// Helper function to cast a sea_orm::DbErr into a domain Database Error.
/// This requires casting the sea_orm::DbErr into anyhow::Error first.
fn to_db_error(e: sea_orm::DbErr) -> scrobblify_domain::errors::DatabaseError {
    scrobblify_domain::errors::DatabaseError::from(anyhow::Error::from(e))
}

fn build_dates_range(opts: ParamsForStatsQuery) -> (NaiveDateTime, NaiveDateTime) {
    let time_start = chrono::NaiveTime::from_hms(0, 0, 0);
    let start = chrono::NaiveDateTime::new(opts.start, time_start);
    let time_end = chrono::NaiveTime::from_hms(23, 59, 59);
    let end = chrono::NaiveDateTime::new(opts.end, time_end);

    (start, end)
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

impl From<ScrobbleQueryResult> for Scrobble {
    fn from(s: ScrobbleQueryResult) -> Self {
        Self {
            timestamp: DateTime::from_str(s.timestamp.as_str()).unwrap(),
            duration_secs: Duration::from_secs_f64(s.duration_secs),
            track: s.track,
            cover: s.cover,
            album: s.album,
            artists: s
                .artists
                .as_str()
                .split(',')
                .into_iter()
                .map(|t| t.to_string())
                .collect(),
            tags: s
                .tags
                .as_str()
                .split(',')
                .into_iter()
                .map(|t| t.to_string())
                .collect(),
        }
    }
}

impl From<PopularTagQueryResult> for StatsTag {
    fn from(t: PopularTagQueryResult) -> Self {
        Self {
            name: t.tag,
            score: t.score,
            listened_secs: t.listened_secs,
        }
    }
}

impl From<PopularTrackQueryResult> for StatsTrack {
    fn from(t: PopularTrackQueryResult) -> Self {
        Self {
            id: t.id,
            title: t.title,
            score: t.score,
            listened_secs: t.listened_secs,
        }
    }
}

impl From<PopularArtistQueryResult> for StatsArtist {
    fn from(t: PopularArtistQueryResult) -> Self {
        Self {
            id: t.id,
            name: t.name,
            score: t.score,
            listened_secs: t.listened_secs,
        }
    }
}
