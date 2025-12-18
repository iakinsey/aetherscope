use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Generic(String),
    #[error(transparent)]
    CdpError(#[from] chromiumoxide::error::CdpError),
    #[error(transparent)]
    BrowserFetcherError(#[from] chromiumoxide::fetcher::FetcherError),
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
