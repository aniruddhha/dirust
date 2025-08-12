//! src/error.rs
//!
//! Central error type for Dirust.
//!
//! Why have our own error enum?
//! - It keeps public function signatures simple: `Result<T, DirustError>`.
//! - It lets us print friendly messages (`Display`) while still keeping debug info (`Debug`).
//! - It allows the `?` operator to convert common error types into `DirustError` via `From`.

use std::{error::Error, fmt};

/// Top-level error type for the application.
///
/// Each variant wraps a concrete error from another library (e.g., `std::io`, `reqwest`),
/// or represents an application-specific condition (e.g., invalid base URL).
#[derive(Debug)] // `Debug` is useful for logs and `{:?}` formatting.
pub enum DirustError {
    /// The provided base URL is invalid for our use:
    /// it must start with `http://` or `https://`.
    InvalidBaseUrl,

    /// Wrapper for file/stream I/O errors (opening wordlist, reading lines, etc.).
    Io(std::io::Error),

    /// Wrapper for HTTP client errors (DNS/TLS/connect/timeouts/protocol) from `reqwest`.
    Http(reqwest::Error),

    /// Header value could not be interpreted as UTF-8 text (`to_str()` failed).
    HeaderToStr(reqwest::header::ToStrError),

    /// An async task failed to join (panic/cancellation surfaced as `JoinError`).
    Join(tokio::task::JoinError),
}

/// Human-readable error messages.
///
/// `Display` is what gets shown to users by default (e.g., when you `println!("{}", err)`).
impl fmt::Display for DirustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirustError::InvalidBaseUrl =>
                write!(f, "base must start with http:// or https://"),

            DirustError::Io(e) =>
                write!(f, "io error: {}", e),

            DirustError::Http(e) =>
                write!(f, "http error: {}", e),

            DirustError::HeaderToStr(e) =>
                write!(f, "header to_str error: {}", e),

            DirustError::Join(e) =>
                write!(f, "task join error: {}", e),
        }
    }
}

/// Implementing `std::error::Error` integrates with the wider error ecosystem:
/// - lets you use `Box<dyn Error>` if you choose
/// - enables source chaining (`source()`) if you add it later
impl Error for DirustError {}

/// Allow `std::io::Error` to be converted into `DirustError::Io` automatically.
///
/// This is what makes the `?` operator work seamlessly in places like:
/// `let f = File::open(path)?;`  // becomes `Err(DirustError::Io(_))` on failure
impl From<std::io::Error> for DirustError {
    fn from(e: std::io::Error) -> Self {
        DirustError::Io(e)
    }
}

/// Convert `reqwest::Error` into `DirustError::Http`.
///
/// Any network/protocol error from `reqwest` can now bubble up with `?`.
impl From<reqwest::Error> for DirustError {
    fn from(e: reqwest::Error) -> Self {
        DirustError::Http(e)
    }
}

/// Convert header UTF-8 conversion errors into `DirustError::HeaderToStr`.
///
/// Used when we call `value.to_str()` on headers like `Content-Length` or `Location`.
impl From<reqwest::header::ToStrError> for DirustError {
    fn from(e: reqwest::header::ToStrError) -> Self {
        DirustError::HeaderToStr(e)
    }
}

/// Convert Tokio task join failures into `DirustError::Join`.
///
/// This surfaces panics/cancellations from spawned tasks back to the caller.
impl From<tokio::task::JoinError> for DirustError {
    fn from(e: tokio::task::JoinError) -> Self {
        DirustError::Join(e)
    }
}
