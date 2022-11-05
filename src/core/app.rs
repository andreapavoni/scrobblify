use anyhow::Result;
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

use super::scrobbler::{Scrobbler, ScrobblerResult};
use super::spotify::SpotifyClient;
use crate::domain::{self, models::Scrobble, repository::Repository};
use crate::domain::{app::App as DomainApp, models::CurrentPlayingTrack};

const SPOTIFY_POLLING_SECS: u64 = 60;

pub struct App {
    db: Box<dyn Repository>,
    spotify: SpotifyClient,
    current_track: CurrentPlayingTrack,
    // TODO: add Last.fm client to get track genres
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
        let track_info = scrobble.track;

        if (self.db.get_track_by_id(track_info.id.clone()).await?).is_none() {
            self.db.insert_track(track_info.clone().into()).await?;
        }

        for artist in track_info.artists.iter() {
            if self.db.get_artist_by_id(artist.id.clone()).await?.is_none() {
                self.db.insert_artist(artist.clone()).await?;
            }
        }

        if self
            .db
            .get_album_by_id(track_info.album.id.clone())
            .await?
            .is_none()
        {
            self.db.insert_album(track_info.album.clone()).await?;
        }

        self.db.insert_scrobble(track_info.clone()).await?;

        Ok(())
    }
}

pub async fn start_scrobbling(app: Arc<Mutex<App>>) -> Result<()> {
    loop {
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

        println!("======= sleep ========");
        let duration = Duration::new(SPOTIFY_POLLING_SECS, 0);
        sleep(duration).await;
    }
}
