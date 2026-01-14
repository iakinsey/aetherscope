use async_trait::async_trait;

use crate::types::{
    configs::frontier_manager_config::FrontierManagerConfig, error::AppError,
    structs::record::Record, traits::task::Task,
};

pub struct FrontierManager<'a> {
    config: &'a FrontierManagerConfig,
}

impl<'a> FrontierManager<'a> {
    pub async fn new(config: &'a FrontierManagerConfig) -> Result<Self, AppError> {
        Ok(Self { config })
    }
}

#[async_trait]
impl<'a> Task for FrontierManager<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        // TODO configurable options for these attributes
        // - Uniqueness filter (bloom filter?)
        // - PageRank
        // - Robots filter

        unimplemented!()
    }
}
