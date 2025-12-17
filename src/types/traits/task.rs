use async_trait::async_trait;

use crate::types::{error::AppError, structs::record::Record};

#[async_trait]
pub trait Task {
    async fn on_message(&self, message: Record) -> Result<Record, AppError>;
}
