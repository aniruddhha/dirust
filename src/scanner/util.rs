//! Small helper functions used across the scanner module:
//!   - `timestamp_seconds()`: produce a UNIX timestamp string for log lines.
//!   - `is_interesting_status()`: decide whether a given HTTP status code is worth printing.
//!
//! We keep these helpers here to avoid cluttering the main scanning logic.

use reqwest::StatusCode;
use std::time::{SystemTime, UNIX_EPOCH};

/// Return the current UNIX timestamp (seconds since 1970-01-01 00:00:00 UTC) as a String.
///
/// Why a string?
///   - Our printing function expects a string to plug into `println!`.
///   - Keeping it as a string avoids repeated conversions at every call site.
///
/// Error handling:
///   - `SystemTime::now().duration_since(UNIX_EPOCH)` can theoretically fail if the system clock
///     is set *before* the epoch. That would indicate a broken clock/config.
///   - We call `.expect(...)` here to crash early with a clear message in that (rare) situation,
///     because continuing would make our output timestamps meaningless.
pub fn timestamp_seconds() -> String {
    // Get "now" according to the OS.
    let now = SystemTime::now()
        // Compute how much time has elapsed since the UNIX epoch.
        .duration_since(UNIX_EPOCH)
        // If the system time is earlier than the epoch, abort with a clear message.
        .expect("system time before UNIX_EPOCH");

    // Format the elapsed seconds as a decimal string, e.g., "1723456789".
    format!("{}", now.as_secs())
}

/// Return `true` if this HTTP status code is considered "interesting" for directory discovery.
///
/// Rationale:
///   - 200 OK: content exists (file or dir index).
///   - 301/302 Moved: often indicates a valid path (e.g., adding a trailing slash).
///   - 401 Unauthorized / 403 Forbidden: strongly suggests the resource exists but is protected.
///
/// Everything else (e.g., 404 Not Found, 500 Internal Server Error) is ignored by default to keep
/// output focused. You can adjust this policy later (e.g., accept 500/405/204) depending on needs.
pub fn is_interesting_status(status: StatusCode) -> bool {
    match status {
        // 200: resource found
        StatusCode::OK
        // 302: Found (temporary redirect)
        | StatusCode::FOUND
        // 301: Moved Permanently (common for directory paths without trailing slash)
        | StatusCode::MOVED_PERMANENTLY
        // 401: requires auth (resource likely exists)
        | StatusCode::UNAUTHORIZED
        // 403: forbidden (resource exists but access denied)
        | StatusCode::FORBIDDEN => true,

        // Any other status code is not “interesting” for our default signal set.
        _ => false,
    }
}
