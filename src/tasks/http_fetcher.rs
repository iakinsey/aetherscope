use crate::types::{
    configs::http_fetcher_config::HttpFetcherConfig, error::AppError, structs::record::Record,
    traits::task::Task,
};
use async_trait::async_trait;

pub struct HttpFetcher<'a> {
    config: &'a HttpFetcherConfig,
}

impl<'a> HttpFetcher<'a> {
    pub async fn new(config: &'a HttpFetcherConfig) -> Result<Self, AppError> {
        unimplemented!()
    }
}

#[async_trait]
impl<'a> Task for HttpFetcher<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
