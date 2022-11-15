use std::time::Duration;

pub fn secs_to_hours_and_minutes(duration: Duration) -> (u64, u64) {
    let duration = duration.as_secs();

    let hours = duration.clone() / 3600;
    let minutes = if hours > 0 {
        duration.clone() % 60
    } else {
        duration.clone() / 60
    };

    (hours, minutes)
}
