//! src/url.rs
//!
//! Purpose:
//!   Validate and normalize the base URL string provided on the CLI.
//!
//! Behavior:
//!   - Accept only `http://` or `https://` schemes (reject anything else).
//!   - Ensure the base ends with a trailing slash `/` so later joins are predictable.
//!
//! Notes / assumptions:
//!   - We treat the input as an opaque string and do minimal checks:
//!       * leading/trailing whitespace is trimmed
//!       * scheme must be http or https
//!       * add a trailing slash if missing
//!   - We do NOT parse or validate hostnames, ports, query strings, or fragments here.
//!     If you later need strict URL parsing/validation, consider the `url` crate.

use crate::error::DirustError;

/// Ensure the base URL starts with http/https and ends with a trailing slash.
///
/// Examples:
///   Input:  "http://example.com"   → Ok("http://example.com/")
///   Input:  "https://x/y/"         → Ok("https://x/y/")
///   Input:  "ftp://example.com"    → Err(InvalidBaseUrl)
///
/// Errors:
///   - Returns `DirustError::InvalidBaseUrl` if the scheme is not http/https.
pub fn normalize_base(base: &str) -> Result<String, DirustError> {
    // Make a new owned String we can modify. We also trim any surrounding whitespace
    // so accidental spaces in the CLI do not break our checks.
    let mut b: String = base.trim().to_string();

    // Explicit booleans for the two allowed schemes.
    // Keeping these as named variables makes the condition below very readable.
    let starts_http: bool = b.starts_with("http://");
    let starts_https: bool = b.starts_with("https://");

    // Reject anything that does not start with `http://` or `https://`.
    // This keeps the tool focused on HTTP(S) and avoids surprising behavior
    // with unsupported schemes (ftp, file, data, etc.).
    if !starts_http && !starts_https {
        return Err(DirustError::InvalidBaseUrl);
    }

    // If the base does not already end with a slash, append one.
    // This guarantees that simple string concatenation later (base + path)
    // does not produce a missing-slash mistake: "https://x/y" + "admin" → "https://x/yadmin"
    if !b.ends_with('/') {
        b.push('/');
    }

    // Return the normalized base string.
    Ok(b)
}
