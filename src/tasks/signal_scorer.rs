use async_trait::async_trait;

use crate::types::{
    configs::tasks::signal_scorer_config::SignalScorerConfig,
    error::AppError,
    structs::{metadata::execution_context::ExecutionContext, record::Record},
    traits::task::Task,
};

pub struct SignalScorer {
    config: SignalScorerConfig,
}

impl SignalScorer {
    pub async fn new(config: SignalScorerConfig) -> Result<Self, AppError> {
        unimplemented!()
    }
}

#[async_trait]
impl Task for SignalScorer {
    async fn on_message(&self, ctx: ExecutionContext, message: Record) -> Result<Record, AppError> {
        unimplemented!()
    }
}
