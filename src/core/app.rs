use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::scrobbler::{Scrobbler, ScrobblerResult};
use crate::bridge::spotify::SpotifyClient;
use crate::domain::{
    self,
    app::App as DomainApp,
    bridge::spotify::SpotifyApi,
    db::Repository,
    models::{CurrentPlayingTrack, Scrobble, Tag},
};

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
                "======= TAGS for ARTIST `{}`: `{:?}`",
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
}

pub async fn auto_scrobble(app: Arc<Mutex<App>>) -> Result<()> {
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
