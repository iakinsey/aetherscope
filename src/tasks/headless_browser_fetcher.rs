use async_trait::async_trait;
use chromiumoxide::{Browser, BrowserConfig};

use crate::types::{error::AppError, structs::record::Record, traits::task::Task};

pub struct HeadlessBrowserFetcher {}

impl HeadlessBrowserFetcher {
    pub async fn new() -> Result<Self, AppError> {
        let (mut browser, mut handler) =
            Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

        unimplemented!()
    }
}

#[async_trait]
impl Task for HeadlessBrowserFetcher {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        unimplemented!()
    }
}
