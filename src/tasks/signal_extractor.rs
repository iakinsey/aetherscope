use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    types::{
        configs::tasks::signal_extractor_config::SignalExtractorConfig,
        error::AppError,
        structs::{
            metadata::execution_context::ExecutionContext, record::Record, signal_base::SignalBase,
        },
        traits::{object_store::ObjectStore, task::Task},
    },
    utils::{cassandra::DbSession, dependencies::dependencies},
};

pub struct SignalExtractor<'a> {
    config: &'a SignalExtractorConfig<'a>,
    object_store: Arc<dyn ObjectStore>,
    db_session: Arc<DbSession>,
}

impl<'a> SignalExtractor<'a> {
    pub async fn new(config: &'a SignalExtractorConfig<'a>) -> Result<Self, AppError> {
        let object_store = dependencies()
            .lock()
            .await
            .get_object_store(&config.object_store)?;

        let db_session = dependencies()
            .lock()
            .await
            .get_db_session(&config.db_session)?;

        Ok(Self {
            config,
            object_store,
            db_session,
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
