use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Generic(String),
    #[error("HTTP {0}: {1}")]
    FetchError(u16, String),
    #[error("{0}")]
    HeadlessBrowserFetcherError(String),
    #[error("Mising dependency: {0}")]
    MissingDependency(String),
    #[error("index out of bounds")]
    IndexOutOfBounds,
    #[error("invalid utf8")]
    InvalidUtf8,
    #[error("parse error: {0}")]
    ParseError(&'static str),
    #[error(transparent)]
    CdpError(#[from] chromiumoxide::error::CdpError),
    #[error(transparent)]
    BrowserFetcherError(#[from] chromiumoxide::fetcher::FetcherError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error("HTTP {method} {status}: {message}")]
    Http {
        status: i64,
        method: String,
        message: String,
    },
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Generic(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Generic(s.to_owned())
    }
}
