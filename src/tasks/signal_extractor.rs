use async_trait::async_trait;

use crate::types::{
    configs::tasks::signal_extractor_config::SignalExtractorConfig, error::AppError,
    structs::record::Record, traits::task::Task,
};

pub struct SignalExtractor<'a> {
    config: &'a SignalExtractorConfig<'a>,
}

impl<'a> SignalExtractor<'a> {
    pub async fn new(config: &'a SignalExtractorConfig<'a>) -> Result<Self, AppError> {
        Ok(Self { config })
    }
}

#[async_trait]
impl<'a> Task for SignalExtractor<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        unimplemented!()
    }
}
