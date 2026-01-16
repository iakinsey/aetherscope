use crate::types::error::AppError;

pub trait FrontierFilter {
    async fn perform(
        self,
        uris: Vec<String>,
        origin: &str,
    ) -> Result<Vec<(String, bool)>, AppError>;
}
