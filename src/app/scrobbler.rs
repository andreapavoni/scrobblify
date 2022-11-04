use chrono::{DateTime, Utc};

use crate::domain::models::{CurrentPlayingTrack, Scrobble, TrackInfo};

// # Spec
// - [ ] As soon as a track has been played for 50% of its length or 4 minutes, it should be counted as a scrobble
// - [ ] That scrobble should be submitted when the play has ended in order to know its duration
// - [ ] If the total play duration is enough to count as a scrobble, but not longer than the total track length + enough for a second scrobble, it should be submitted as a scrobble with the according duration
// - [ ] If the duration exceeds this value, the first scrobble should be submitted as a scrobble with the duration of the full track length, while the second scrobble is queued up following the above suggestions in regards to remaining time
//
// ## Example
// The user starts playing '(Fine Layers of) Slaysenflite', which is exactly 3:00 minutes long.
// - If the user ends the play after 1:22, no scrobble is submitted
// - If the user ends the play after 2:06, a scrobble with `"duration":126` is submitted
// - If the user jumps back several times and ends the play after 3:57, a scrobble with `"duration":237` is submitted
// - If the user jumps back several times and ends the play after 4:49, two scrobbles with `"duration":180` and `"duration":109` are submitted

const SCROBBLE_LISTENING_MIN_SECS: u64 = 180;

pub enum ScrobblerResult {
    Ok(Scrobble),
    Cache,
    Ignore,
}

pub struct Scrobbler {}

impl Scrobbler {
    pub fn calculate_scrobble(
        current: &CurrentPlayingTrack,
        cache: &CurrentPlayingTrack,
    ) -> ScrobblerResult {
        match (current.track.clone(), cache.track.as_ref()) {
            // maybe track has been playing for enough time, so we scrobble it
            (Some(_), Some(_)) => {
                let timestamp = get_timestamp(&current, &cache);
                if let Some(duration) = calculate_duration(&current, timestamp) {
                    return ScrobblerResult::Ok(do_build_scrobble(
                        duration,
                        timestamp,
                        current.track.clone().unwrap(),
                    ));
                }
                return ScrobblerResult::Ignore;
            }
            // the track has been playing for less than enough, let's keep it in memory
            // and maybe it will be eventually scrobbled later
            (Some(_), None) => return ScrobblerResult::Cache,
            // nothing is currently playing, move on
            _ => return ScrobblerResult::Ignore,
        }
    }
}

fn get_timestamp(current: &CurrentPlayingTrack, cache: &CurrentPlayingTrack) -> DateTime<Utc> {
    if current.track.clone().unwrap().id == cache.track.clone().unwrap().id {
        cache.timestamp.unwrap()
    } else {
        current.timestamp.unwrap()
    }
}

fn do_build_scrobble(duration: u64, timestamp: DateTime<Utc>, track: TrackInfo) -> Scrobble {
    Scrobble {
        timestamp,
        duration_secs: duration as f64,
        track: track.clone(),
    }
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
