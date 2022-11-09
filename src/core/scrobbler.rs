use chrono::{DateTime, Utc};

use crate::domain::models::{CurrentPlayingTrack, Scrobble};

const SCROBBLE_LISTENING_MIN_SECS: u64 = 180;

pub enum ScrobblerResult {
    Ok(Scrobble),
    Cache,
    Ignore,
    AlreadyScrobbled,
}

pub struct Scrobbler {}

impl Scrobbler {
    pub fn calculate_scrobble(
        current: &CurrentPlayingTrack,
        cache: &CurrentPlayingTrack,
    ) -> ScrobblerResult {
        if current == cache && cache.scrobbled {
            return ScrobblerResult::AlreadyScrobbled;
        }
        match (current.track.clone(), cache.track.as_ref()) {
            // maybe track has been playing for enough time, so we scrobble it
            (Some(current_track), Some(_)) => {
                let timestamp = get_timestamp(current, cache);
                if let Some(duration) = calculate_duration(current, timestamp) {
                    return ScrobblerResult::Ok(Scrobble {
                        timestamp,
                        duration_secs: duration as f64,
                        track: current_track,
                    });
                }

                ScrobblerResult::Cache
            }
            // the track has been playing for less than enough, let's keep it in memory
            // and maybe it will be eventually scrobbled later
            (Some(_), None) => ScrobblerResult::Cache,
            // nothing is currently playing, move on
            _ => ScrobblerResult::Ignore,
        }
    }
}

fn get_timestamp(current: &CurrentPlayingTrack, cache: &CurrentPlayingTrack) -> DateTime<Utc> {
    let current_timestamp = current.timestamp.unwrap();
    let cache_timestamp = cache.timestamp.unwrap();

    if current == cache {
        return cache_timestamp;
    }
    current_timestamp
}

fn calculate_duration(current: &CurrentPlayingTrack, timestamp: DateTime<Utc>) -> Option<u64> {
    let now = Utc::now();
    let listened_time = now
        .signed_duration_since(timestamp)
        .to_std()
        .unwrap()
        .as_secs();

    let duration = current.track.as_ref().unwrap().duration_secs.as_secs();
    if listened_time >= (duration / 2) || listened_time >= SCROBBLE_LISTENING_MIN_SECS {
        return Some(duration);
    }

    None
}
