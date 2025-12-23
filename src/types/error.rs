use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Generic(String),
    #[error("{0}")]
    HeadlessBrowserFetcherError(String),
    #[error("Mising dependency: {0}")]
    MissingDependency(String),
    #[error(transparent)]
    CdpError(#[from] chromiumoxide::error::CdpError),
    #[error(transparent)]
    BrowserFetcherError(#[from] chromiumoxide::fetcher::FetcherError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),
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
