use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    types::{
        configs::tasks::signal_extractor_config::SignalExtractorConfig,
        error::AppError,
        structs::{
            metadata::execution_context::ExecutionContext, record::Record, signal_base::SignalBase,
        },
        traits::{object_store::ObjectStore, signal::Signal, task::Task},
    },
    utils::dependencies::dependencies,
};

pub struct SignalExtractor<'a> {
    config: &'a SignalExtractorConfig<'a>,
    object_store: Arc<dyn ObjectStore>,
}

impl<'a> SignalExtractor<'a> {
    pub async fn new(config: &'a SignalExtractorConfig<'a>) -> Result<Self, AppError> {
        let object_store = dependencies()
            .lock()
            .await
            .get_object_store(&config.object_store)?;

        Ok(Self {
            config,
            object_store,
        })
    }
}

#[async_trait]
impl<'a> Task for SignalExtractor<'a> {
    async fn on_message(&self, ctx: ExecutionContext, message: Record) -> Result<Record, AppError> {
        //let mut signals: Vec<dyn Signal> = vec![];
        let signal_base = SignalBase::new(&message)?;

        unimplemented!()
    }
}
