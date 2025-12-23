use crate::types::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait ObjectStore {
    async fn get(&self, key: &str) -> Result<Vec<u8>, AppError>;
    async fn put(&self, key: &str, data: &[u8]) -> Result<(), AppError>;
    async fn delete(&self, key: &str) -> Result<(), AppError>;
    async fn exists(&self, key: &str) -> Result<bool, AppError>;
}
