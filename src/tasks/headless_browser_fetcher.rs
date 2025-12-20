use std::{path::PathBuf, sync::Arc};

use crate::{
    types::{error::AppError, structs::record::Record, traits::task::Task},
    utils::{fs::download_browser, sync::TabPool},
};
use async_trait::async_trait;
use chromiumoxide::{Browser, BrowserConfig, Handler, Page, browser, handler::http};
use fastpool::bounded::{Pool, PoolConfig};
use futures::StreamExt;
use tokio::{spawn, task::JoinHandle};

pub struct HeadlessBrowserConfig {
    http_proxy: Option<String>,
    browser_path: Option<String>,
}

pub struct HeadlessBrowserFetcher {
    browser: Arc<Browser>,
    _handle: JoinHandle<()>,
    pool: Arc<Pool<TabPool>>,
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

        let (browser, mut handler) = Browser::launch(browser_config.build()?).await?;
        let browser = Arc::new(browser);

        let _handle = spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        let pool = Pool::new(PoolConfig::new(16), TabPool::new(Arc::clone(&browser)));

        Ok(Self {
            browser,
            _handle,
            pool,
        })
    }
}

#[async_trait]
impl Task for HeadlessBrowserFetcher {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        let page = self.pool.get().await?;
        unimplemented!();
    }
}
