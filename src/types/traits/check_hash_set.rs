use async_trait::async_trait;

use crate::types::error::AppError;

#[async_trait]
pub trait CheckHashSet {
    async fn contains_entities(
        &self,
        entities: Vec<String>,
    ) -> Result<Vec<(String, bool)>, AppError>;
}
