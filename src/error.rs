use std::{error::Error, fmt};

#[derive(Debug)]
pub enum DirustError {
    InvalidBaseUrl,
    Io(std::io::Error),
    Http(reqwest::Error),
    HeaderToStr(reqwest::header::ToStrError),
    Join(tokio::task::JoinError),
}

impl fmt::Display for DirustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirustError::InvalidBaseUrl => write!(f, "base must start with http:// or https://"),
            DirustError::Io(e) => write!(f, "io error: {}", e),
            DirustError::Http(e) => write!(f, "http error: {}", e),
            DirustError::HeaderToStr(e) => write!(f, "header to_str error: {}", e),
            DirustError::Join(e) => write!(f, "task join error: {}", e),
        }
    }
}
impl Error for DirustError {}

impl From<std::io::Error> for DirustError {
    fn from(e: std::io::Error) -> Self { DirustError::Io(e) }
}
impl From<reqwest::Error> for DirustError {
    fn from(e: reqwest::Error) -> Self { DirustError::Http(e) }
}
impl From<reqwest::header::ToStrError> for DirustError {
    fn from(e: reqwest::header::ToStrError) -> Self { DirustError::HeaderToStr(e) }
}
impl From<tokio::task::JoinError> for DirustError {
    fn from(e: tokio::task::JoinError) -> Self { DirustError::Join(e) }
}