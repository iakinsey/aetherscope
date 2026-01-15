use crate::types::error::AppError;

pub trait FrontierFilter {
    async fn filter(uris: Vec<String>, origin: &str) -> Result<bool, AppError>;
}
