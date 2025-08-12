
//!
//! Responsibilities of this module:
//!   1) Send a single HTTP request to a target URL (HEAD by default).
//!   2) Fall back to GET when HEAD is not allowed (405 Method Not Allowed).
//!   3) Extract just the fields the scanner prints: status, Content-Length, Location.
//!
//! Design choices (important for understanding):
//!   - We do NOT follow redirects automatically. Seeing 30x + Location is useful during discovery.
//!   - We do NOT read the response body here; HEAD avoids body, and for GET we still skip the body.
//!     Skipping bodies keeps the tool fast and light for enumeration.
//!   - We keep error handling explicit and convert external errors into `DirustError`.
//!   - We only include header values that are valid UTF-8; otherwise we treat them as missing.

use crate::error::DirustError;
use reqwest::{header, Client, Response, StatusCode};

/// A minimal summary of an HTTP response that the scanner knows how to print.
///
/// Fields:
/// - `status`:           The HTTP status code (e.g., 200, 301, 403).
/// - `content_length`:   `Some("<number>")` if the `Content-Length` header exists and is valid UTF-8; otherwise `None`.
/// - `location`:         `Some("<url>")` if the `Location` header exists and is valid UTF-8; otherwise `None`.
///
/// Note: We intentionally keep this struct small—just enough for meaningful CLI output.
#[derive(Debug)]
pub struct HttpSummary {
    pub status: StatusCode,
    pub content_length: Option<String>,
    pub location: Option<String>,
}

/// Convert a full `reqwest::Response` into our compact `HttpSummary`.
///
/// What we keep:
///   - Status code
///   - `Content-Length` header (if present + valid UTF-8)
///   - `Location` header (if present + valid UTF-8)
///
/// What we ignore (on purpose):
///   - The response body (to keep scans fast)
///   - Other headers (not needed for basic directory busting)
fn summarize_response(resp: Response) -> HttpSummary {
    // Attempt to read Content-Length from headers.
    // If the header value is not valid UTF-8, we ignore it to avoid printing garbage.
    let len_opt: Option<String> = match resp.headers().get(header::CONTENT_LENGTH) {
        Some(v) => match v.to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => None, // Non-UTF8 header → treat as absent
        },
        None => None, // Header not present
    };

    // Attempt to read Location from headers.
    // This is typically present on 30x responses and is useful to show redirect targets.
    let loc_opt: Option<String> = match resp.headers().get(header::LOCATION) {
        Some(v) => match v.to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => None, // Non-UTF8 header → treat as absent
        },
        None => None, // No Location header
    };

    HttpSummary {
        status: resp.status(),
        content_length: len_opt,
        location: loc_opt,
    }
}

/// Send one HTTP request and return a summarized response.
///
/// Parameters:
/// - `client`:  A pre-built `reqwest::Client` (shared across tasks to reuse connections).
/// - `url`:     The absolute URL to probe.
/// - `use_get`: If `true`, send a GET immediately. If `false`, try HEAD first for speed.
///
/// Behavior:
/// - Default (HEAD first): We prefer HEAD because it typically avoids downloading bodies.
/// - Fallback: If the server returns `405 Method Not Allowed` to HEAD, we retry the same URL with GET.
/// - We do not follow redirects; we want to *see* them (status + Location).
///
/// Returns:
/// - `Ok(HttpSummary)` on success, containing status/headers of interest.
/// - `Err(DirustError)` on network/protocol errors (DNS, TLS, socket, etc.).
pub async fn probe(client: &Client, url: &str, use_get: bool) -> Result<HttpSummary, DirustError> {
    // Decide the initial method:
    // - GET if the caller asked for it (some servers misbehave on HEAD).
    // - Otherwise HEAD, which is faster and avoids body downloads where supported.
    let mut response_result = if use_get {
        client.get(url).send().await
    } else {
        client.head(url).send().await
    };

    // If the first request succeeded but came back with 405 (Method Not Allowed),
    // and we *did not* force GET, then retry with GET to be robust.
    match &response_result {
        Ok(resp) => {
            if resp.status() == StatusCode::METHOD_NOT_ALLOWED && use_get == false {
                // A number of servers or frameworks may not implement HEAD properly.
                // Doing a second attempt with GET makes the tool more compatible.
                response_result = client.get(url).send().await;
            }
        }
        Err(_) => {
            // If the request failed (DNS/TLS/timeout/connection), we do nothing here;
            // the error is handled below when we unwrap `response_result`.
        }
    }

    // Turn the `Result<Response, reqwest::Error>` into either a `Response` or our error type.
    let response = match response_result {
        Ok(r) => r,
        Err(e) => return Err(DirustError::from(e)),
    };

    // Reduce the response down to the key printable fields.
    let summary = summarize_response(response);
    Ok(summary)
}
