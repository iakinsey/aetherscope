use std::path::PathBuf;

use async_trait::async_trait;
use chromiumoxide::{Browser, BrowserConfig, Handler, browser, handler::http};

use crate::{
    types::{error::AppError, structs::record::Record, traits::task::Task},
    utils::fs::download_browser,
};

pub struct HeadlessBrowserConfig {
    http_proxy: Option<String>,
    browser_path: Option<String>,
}

pub struct HeadlessBrowserFetcher {
    browser: Browser,
    handler: Handler,
}

impl HeadlessBrowserFetcher {
    pub async fn new(config: &HeadlessBrowserConfig) -> Result<Self, AppError> {
        let browser_path = match &config.browser_path {
            Some(p) => PathBuf::from(p),
            None => download_browser(None).await?,
        };

        let mut browser_config = BrowserConfig::builder()
            .with_head()
            .chrome_executable(browser_path);

        if let Some(http_proxy) = &config.http_proxy {
            browser_config = browser_config.arg(format!("--proxy-server={}", http_proxy))
        }

        let (browser, handler) = Browser::launch(browser_config.build()?).await?;

        Ok(Self { browser, handler })
    }
}

#[async_trait]
impl Task for HeadlessBrowserFetcher {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        unimplemented!()
    }
}
