use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

use super::App;
use scrobblify_domain::{
    app::App as DomainApp,
    models::{CurrentPlayingTrack, ScrobbleInfo},
};

const SCROBBLE_LISTENING_MIN_SECS: u64 = 180;
const SPOTIFY_POLLING_SECS: u64 = 60;

pub enum ScrobblerResult {
    Ok(ScrobbleInfo),
    Cache,
    NotPlaying,
    AlreadyScrobbled,
    NotReadyForScrobble,
}

pub struct Scrobbler;

impl Scrobbler {
    pub async fn start_auto_scrobbling(app: Arc<Mutex<App>>) {
        tokio::spawn(async move {
            tracing::info!(msg = "start auto-scrobbling");

            loop {
                if let Err(err) = Self::auto_scrobble(app.clone()).await {
                    tracing::error!(msg = "error while scrobbling", error = format!("{:?}", err))
                }
                let duration = Duration::new(SPOTIFY_POLLING_SECS, 0);
                sleep(duration).await;
            }
        });
    }

    pub async fn scrobble_recently_played(app: Arc<Mutex<App>>) {
        tracing::info!(msg = "check recently played tracks");
        if let Ok(recently_played) = app.lock().await.get_recently_played().await {
            for played in recently_played {
                let scrobble = ScrobbleInfo {
                    timestamp: played.played_at,
                    duration_secs: played.track.duration_secs.as_secs_f64(),
                    track: played.track,
                };

                log_scrobbling(&scrobble.clone(), "scrobble recently played");
                let _ = app.lock().await.scrobble(scrobble).await;
            }
        };
    }

    async fn auto_scrobble(app: Arc<Mutex<App>>) -> Result<()> {
        let mut app = app.lock().await;
        let current = &app.get_currently_playing().await?;
        let cache = app.get_current_track();

        match calculate_scrobble(current, cache) {
            ScrobblerResult::Ok(scrobble) => {
                let mut new_current = current.clone().unwrap();
                new_current.scrobbled = true;

                log_scrobbling(&scrobble.clone(), "scrobble");
                app.scrobble(scrobble).await?;
                app.set_current_track(Some(new_current.clone()));
            }
            ScrobblerResult::Cache => {
                let new_current = current.clone().unwrap();
                app.set_current_track(Some(new_current.clone()));

                let title = new_current.clone().track.title;
                tracing::debug!(msg = "scrobble: cache track", title = title,);
            }
            ScrobblerResult::NotPlaying => {
                app.set_current_track(None);
                tracing::debug!(msg = "scrobble: nothing is playing");
            }
            ScrobblerResult::AlreadyScrobbled => {
                tracing::debug!(msg = "scrobble: skip already scrobbled");
            }
            ScrobblerResult::NotReadyForScrobble => {
                tracing::debug!(msg = "scrobble: not ready yet");
            }
        };

        Ok(())
    }
}

fn calculate_scrobble(
    current: &Option<CurrentPlayingTrack>,
    cache: &Option<CurrentPlayingTrack>,
) -> ScrobblerResult {
    match (current, cache) {
        // track has been playing for enough time, so we scrobble it
        (Some(current), Some(cache)) => {
            // the current playing track hasn't been listened enough, cache for later
            if *current != *cache {
                return ScrobblerResult::Cache;
            }

            // already scrobbled, skip
            if cache.scrobbled {
                return ScrobblerResult::AlreadyScrobbled;
            }

            let timestamp = get_timestamp(current, cache);
            if let Some(duration) = calculate_duration(&current, timestamp) {
                // the track has been playing for enough, scrobble it
                return ScrobblerResult::Ok(ScrobbleInfo {
                    timestamp,
                    duration_secs: duration as f64,
                    track: current.clone().track,
                });
            }
            // the track hasn't been playing for enough, skip for later
            ScrobblerResult::NotReadyForScrobble
        }
        // the first playing track has been hasn't been listened enough, cache for later
        (Some(_), None) => ScrobblerResult::Cache,
        // nothing is currently playing, skip
        _ => ScrobblerResult::NotPlaying,
    }
}

fn get_timestamp(current: &CurrentPlayingTrack, cache: &CurrentPlayingTrack) -> DateTime<Utc> {
    if *current == *cache {
        return cache.clone().timestamp;
    }
    current.clone().timestamp
}

fn calculate_duration(current: &CurrentPlayingTrack, timestamp: DateTime<Utc>) -> Option<u64> {
    let now = Utc::now();
    let listened_time = now
        .signed_duration_since(timestamp)
        .to_std()
        .unwrap_or_else(|err| {
            tracing::error!(
                msg = "scrobbler:calculate_duration",
                timestamp = format!("{:?}", timestamp),
                now = format!("{:?}", now),
                error = format!("{:?}", err)
            );
            return Duration::new(0, 0);
        })
        .as_secs();

    let duration = current.track.clone().duration_secs.as_secs();
    if listened_time >= (duration / 2) || listened_time >= SCROBBLE_LISTENING_MIN_SECS {
        return Some(duration);
    }

    None
}

fn log_scrobbling(scrobble: &ScrobbleInfo, msg: &str) {
    let title = scrobble.clone().track.title;
    let artists = scrobble
        .clone()
        .track
        .artists
        .into_iter()
        .map(|a| a.name)
        .collect::<Vec<String>>()
        .join(", ");

    tracing::info!(msg, title = title, artists = artists,);
}
