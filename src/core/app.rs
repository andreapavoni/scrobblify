use anyhow::Result;

use crate::bridge::spotify::SpotifyClient;
use crate::domain::{
    self,
    bridge::spotify::SpotifyApi,
    db::Repository,
    models::{CurrentPlayingTrack, Scrobble},
};

pub struct App {
    pub current_track: CurrentPlayingTrack,
    db: Box<dyn Repository>,
    pub spotify: SpotifyClient,
}

impl App {
    pub fn new(db: Box<dyn Repository>, spotify: SpotifyClient) -> Self {
        App {
            db,
            spotify,
            current_track: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl domain::app::App for App {
    async fn scrobble(&self, scrobble: Scrobble) -> Result<()> {
        let mut track_info = scrobble.track;

        self.db.insert_track(track_info.clone().into()).await?;

        let mut artists_ids: Vec<&str> = vec![];
        for artist in track_info.artists.iter() {
            // TODO: if an artist is already on db, maybe we don't need to fetch tags
            // same for track
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
        self.db.insert_scrobble(track_info.clone()).await?;

        Ok(())
    }

    async fn scrobble_recently_played(&self) -> Result<()> {
        if let Some(timestamp) = self.db.get_last_scrobble_timestamp().await? {
            let recently_played = self.spotify.get_recently_played(timestamp).await?;

            for played in recently_played {
                let scrobble = Scrobble {
                    timestamp: played.played_at,
                    duration_secs: played.track.duration_secs.as_secs_f64(),
                    track: played.track,
                };

                self.scrobble(scrobble).await?;
            }
        }

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
