use std::sync::Arc;

use async_trait::async_trait;

use crate::types::{
    configs::url_extractor_config::UrlExtractorConfig,
    error::AppError,
    structs::record::Record,
    traits::{object_store::ObjectStore, task::Task},
};

pub struct UrlExtractor<'a> {
    config: &'a UrlExtractorConfig,
    object_store: Arc<dyn ObjectStore>,
}

impl<'a> UrlExtractor<'a> {
    pub async fn new(config: &'a UrlExtractorConfig) -> Result<Self, AppError> {
        unimplemented!();
    }
}

// TODO use spawn_blocking
#[async_trait]
impl<'a> Task for UrlExtractor<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        unimplemented!();
    }
}
