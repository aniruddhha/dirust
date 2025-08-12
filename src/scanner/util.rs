use reqwest::StatusCode;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp_seconds() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH");
    format!("{}", now.as_secs())
}

pub fn is_interesting_status(status: StatusCode) -> bool {
    match status {
        StatusCode::OK
        | StatusCode::FOUND
        | StatusCode::MOVED_PERMANENTLY
        | StatusCode::UNAUTHORIZED
        | StatusCode::FORBIDDEN => true,
        _ => false,
    }
}