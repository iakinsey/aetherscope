use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    types::{
        error::AppError,
        structs::record::{Record, RecordMetadata},
        traits::task::Task,
    },
    utils::{fs::download_browser, sync::TabPool},
};
use async_trait::async_trait;
use chromiumoxide::cdp::browser_protocol::network::RequestId;
use chromiumoxide::{Browser, BrowserConfig, Handler, Page, browser, handler::http};
use fastpool::bounded::{Pool, PoolConfig};
use futures::StreamExt;
use serde_json::Value;
use tokio::{
    spawn,
    sync::{Mutex, oneshot::Receiver},
    task::JoinHandle,
};

pub struct HeadlessBrowserConfig {
    http_proxy: Option<String>,
    browser_path: Option<String>,
    enable_http_metrics: bool,
}

#[derive(Debug, Clone)]
struct ReqInfo {
    method: String,
    req_headers: serde_json::Value,
    t0: f64,
}

#[derive(Debug, Clone)]
struct RespInfo {
    status: i64,
    resp_headers: Value,
}

struct HttpMetricsRx {
    req_map: Arc<Mutex<HashMap<RequestId, ReqInfo>>>,
    resp_map: Arc<Mutex<HashMap<RequestId, RespInfo>>>,
    done_rx: Receiver<(RequestId, f64)>,
}

pub struct HeadlessBrowserFetcher<'a> {
    browser: Arc<Browser>,
    _handle: JoinHandle<()>,
    pool: Arc<Pool<TabPool>>,
    config: &'a HeadlessBrowserConfig,
}

impl<'a> HeadlessBrowserFetcher<'a> {
    pub async fn new(config: &'a HeadlessBrowserConfig) -> Result<Self, AppError> {
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

        let pool = Pool::new(
            PoolConfig::new(16),
            TabPool::new(Arc::clone(&browser), config.enable_http_metrics),
        );

        Ok(Self {
            browser,
            _handle,
            pool,
            config,
        })
    }

    async fn install_http_metrics(&self, page: &Page) -> Result<HttpMetricsRx, AppError> {
        unimplemented!();
    }
}

#[async_trait]
impl<'a> Task for HeadlessBrowserFetcher<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        let tab = self.pool.get().await?;
        let page = tab.goto(message.uri).await?;
        let html = page.wait_for_navigation().await?.content().await?;
        let mut metadata = vec![];
        let metrics = self.install_http_metrics(page).await?;

        unimplemented!()
    }
}
