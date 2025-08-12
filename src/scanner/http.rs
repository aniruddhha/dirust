use crate::error::DirustError;
use reqwest::{header, Client, Response, StatusCode};

#[derive(Debug)]
pub struct HttpSummary {
    pub status: StatusCode,
    pub content_length: Option<String>,
    pub location: Option<String>,
}

fn summarize_response(resp: Response) -> HttpSummary {
    // Content-Length (string form if available and UTF-8)
    let len_opt: Option<String> = match resp.headers().get(header::CONTENT_LENGTH) {
        Some(v) => match v.to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => None,
        },
        None => None,
    };

    // Location header (string if valid UTF-8)
    let loc_opt: Option<String> = match resp.headers().get(header::LOCATION) {
        Some(v) => match v.to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => None,
        },
        None => None,
    };

    HttpSummary {
        status: resp.status(),
        content_length: len_opt,
        location: loc_opt,
    }
}

pub async fn probe(client: &Client, url: &str, use_get: bool) -> Result<HttpSummary, DirustError> {
    // Try HEAD unless user forces GET. If HEAD is 405, retry with GET.
    let mut response_result = if use_get {
        client.get(url).send().await
    } else {
        client.head(url).send().await
    };

    match &response_result {
        Ok(resp) => {
            if resp.status() == StatusCode::METHOD_NOT_ALLOWED && use_get == false {
                response_result = client.get(url).send().await;
            }
        }
        Err(_) => { /* handled below */ }
    }

    let response = match response_result {
        Ok(r) => r,
        Err(e) => return Err(DirustError::from(e)),
    };

    let summary = summarize_response(response);
    Ok(summary)
}

