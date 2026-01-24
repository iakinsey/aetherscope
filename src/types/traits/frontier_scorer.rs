use crate::types::error::AppError;

pub trait FrontierScorer {
    async fn score(self, uris: Vec<String>, origin: &str) -> Result<Vec<(String, i32)>, AppError>;
}
