use async_trait::async_trait;

use crate::types::{error::AppError, structs::record::Record, traits::task::Task};

pub struct HeadlessBrowserFetcher {}

impl HeadlessBrowserFetcher {
    pub fn new() -> Result<Self, AppError> {
        unimplemented!()
    }
}

#[async_trait]
impl Task for HeadlessBrowserFetcher {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        unimplemented!()
    }
}
