use anyhow::Result;

use crate::bridge::spotify::SpotifyClient;
use crate::domain::{
    self,
    bridge::spotify::SpotifyApi,
    db::Repository,
    models::{CurrentPlayingTrack, Scrobble, Tag},
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

        let mut all_tags: Vec<Tag> = vec![];

        for artist in track_info.artists.iter() {
            // TODO: if an artist is already on db, maybe we don't need to fetch tags
            // same for track
            self.db.insert_artist(artist.clone()).await?;

            // fetching genres from the artist profile, it's the most reliable way to get some tags
            let tags = self.spotify.get_tags(&artist.id).await?;
            println!(
                "TAGS for ARTIST `{}`: `{:?}`",
                artist.clone().name,
                tags.clone()
            );
            for tag in tags.iter() {
                self.db.insert_tag(tag.clone()).await?;
                all_tags.push(tag.clone());
            }
        }

        track_info.tags = all_tags.clone();
        track_info.tags.sort();
        track_info.tags.dedup();

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
