use anyhow::Result;
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

use super::scrobbler::{Scrobbler, ScrobblerResult};
use crate::bridge::spotify::SpotifyClient;
use crate::domain::{
    self,
    app::App as DomainApp,
    bridge::spotify::SpotifyApi,
    db::Repository,
    models::{CurrentPlayingTrack, Scrobble, Tag},
};

const SPOTIFY_POLLING_SECS: u64 = 60;

pub struct App {
    current_track: CurrentPlayingTrack,
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

pub async fn start_auto_scrobbling(app: Arc<Mutex<App>>) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = auto_scrobble(app.clone()).await {
                tracing::error!("error while scrobbling: `{:?}`", err)
            }

            println!("======= sleep ========");
            let duration = Duration::new(SPOTIFY_POLLING_SECS, 0);
            sleep(duration).await;
        }
    });
}

async fn auto_scrobble(app: Arc<Mutex<App>>) -> Result<()> {
    let mut app = app.lock().await;
    let current = app.spotify.get_currently_playing().await?;
    let cache = app.current_track.clone();

    match Scrobbler::calculate_scrobble(&current, &cache) {
        ScrobblerResult::Ok(scrobble) => {
            app.current_track = current.clone();
            println!("======= new scrobble: `{:#?}` ========", scrobble.clone());
            app.scrobble(scrobble).await?;
            app.current_track.scrobbled = true;
        }
        ScrobblerResult::Cache => {
            app.current_track = current.clone();
            println!("======= cache track: `{:#?}` ========", current.clone());
        }
        ScrobblerResult::Ignore => {
            app.current_track = Default::default();
            println!("======= ignore ========");
        }
        ScrobblerResult::AlreadyScrobbled => {
            println!("======= already scrobbled ========");
        }
    };

    Ok(())
}
