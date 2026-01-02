use crate::types::error::AppError;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;
use tokio::io::{AsyncRead, AsyncSeek};

pub trait AsyncReadSeek: AsyncRead + AsyncSeek {}
impl<T: AsyncRead + AsyncSeek + ?Sized> AsyncReadSeek for T {}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Vec<u8>, AppError>;
    async fn put(&self, key: &str, data: &[u8]) -> Result<(), AppError>;
    async fn put_stream(
        &self,
        key: &str,
        stream: BoxStream<'_, Result<Bytes, AppError>>,
    ) -> Result<(), AppError>;
    async fn get_stream(
        &self,
        key: &str,
    ) -> Result<Box<dyn AsyncReadSeek + Send + Unpin>, AppError>;
    async fn delete(&self, key: &str) -> Result<(), AppError>;
}
