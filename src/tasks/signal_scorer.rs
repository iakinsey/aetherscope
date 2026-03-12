use async_trait::async_trait;

use crate::types::{
    error::AppError,
    structs::{metadata::execution_context::ExecutionContext, record::Record},
    traits::task::Task,
};

pub struct SignalScorer {}

#[async_trait]
impl Task for SignalScorer {
    async fn on_message(&self, ctx: ExecutionContext, message: Record) -> Result<Record, AppError> {
        unimplemented!()
    }
}
