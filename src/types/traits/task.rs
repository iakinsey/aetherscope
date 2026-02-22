use async_trait::async_trait;

use crate::types::{
    error::AppError,
    structs::{metadata::execution_context::ExecutionContext, record::Record},
};

#[async_trait]
pub trait Task {
    async fn on_message(&self, ctx: ExecutionContext, message: Record) -> Result<Record, AppError>;
}
