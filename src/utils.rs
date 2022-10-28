use std::time::Duration;

pub fn format_time_from_duration(duration: Duration) -> String {
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;
    format!("{}:{}", minutes, seconds)
}
