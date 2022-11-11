use anyhow::Result;

use crate::bridge::spotify::SpotifyClient;
use crate::domain::models::HistoryPlayedTrack;
use crate::domain::{
    self,
    bridge::spotify::SpotifyApi,
    db::Repository,
    models::{CurrentPlayingTrack, ScrobbleInfo},
};

pub struct App {
    current_track: Option<CurrentPlayingTrack>,
    db: Box<dyn Repository>,
    spotify: SpotifyClient,
}

impl App {
    pub fn new(db: Box<dyn Repository>, spotify: SpotifyClient) -> Self {
        App {
            db,
            spotify,
            current_track: Default::default(),
        }
    }

    pub fn get_current_track(&self) -> &Option<CurrentPlayingTrack> {
        &self.current_track
    }

    pub fn set_current_track(&mut self, current_track: Option<CurrentPlayingTrack>) {
        self.current_track = current_track;
    }
}

#[async_trait::async_trait]
impl domain::app::App for App {
    async fn get_recently_played(&self) -> Result<Vec<HistoryPlayedTrack>> {
        if let Some(scrobble) = self.db.get_last_scrobble().await? {
            let recently_played = self.spotify.get_recently_played(scrobble.timestamp).await?;

            return Ok(recently_played);
        }
        Ok(vec![])
    }

    async fn get_currently_playing(&self) -> Result<Option<CurrentPlayingTrack>> {
        self.spotify.get_currently_playing().await
    }

    async fn scrobble(&self, scrobble: ScrobbleInfo) -> Result<()> {
        let mut track_info = scrobble.track;

        // if a track is already on db, we don't need to fetch tags and insert stuff on db again
        if let Ok(None) = self.db.get_track_by_id(track_info.clone().id).await {
            self.db.insert_track(track_info.clone().into()).await?;
            let mut artists_ids: Vec<&str> = vec![];
            for artist in track_info.artists.iter() {
                self.db.insert_artist(artist.clone()).await?;
                artists_ids.push(&artist.id);
            }

            // fetching genres from the artist profile, it's the most reliable way to get some tags
            let tags = self.spotify.get_tags(artists_ids).await?;
            for tag in tags.iter() {
                self.db.insert_tag(tag.clone()).await?;
            }
            track_info.tags = tags.clone();

            self.db.insert_album(track_info.album.clone()).await?;
        }
        self.db.insert_scrobble(track_info.clone()).await?;

        Ok(())
    }

    fn is_spotify_authenticated(&self) -> bool {
        self.spotify.has_auth()
    }

    async fn get_spotify_auth_url(&self) -> Result<String> {
        self.spotify.get_auth_url().await
    }

    async fn store_spotify_auth_token(&self, code: &str) -> Result<()> {
        self.spotify.clone().get_auth_token(code).await
    }
}
