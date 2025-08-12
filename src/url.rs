use crate::error::DirustError;

/// Ensure the base URL starts with http/https and ends with a trailing slash.
/// Example inputs:
///   "http://example.com"      -> Ok("http://example.com/")
///   "https://x/y/"            -> Ok("https://x/y/")
///   "ftp://example.com"       -> Err(InvalidBaseUrl)
pub fn normalize_base(base: &str) -> Result<String, DirustError> {
    let mut b: String = base.trim().to_string();

    let starts_http: bool = b.starts_with("http://");
    let starts_https: bool = b.starts_with("https://");

    if !starts_http && !starts_https {
        return Err(DirustError::InvalidBaseUrl);
    }

    if !b.ends_with('/') {
        b.push('/');
    }

    Ok(b)
}