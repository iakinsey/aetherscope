use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    types::{error::AppError, structs::record::Record, traits::task::Task},
    utils::{fs::download_browser, sync::TabPool},
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chromiumoxide::cdp::browser_protocol::network;
use chromiumoxide::{Browser, BrowserConfig, Page};
use fastpool::bounded::{Pool, PoolConfig};
use futures::StreamExt;
use tokio::{spawn, task::JoinHandle};

pub struct HeadlessBrowserConfig {
    http_proxy: Option<String>,
    browser_path: Option<String>,
    enable_http_metrics: bool,
}

#[derive(Debug, Clone)]
struct HttpRequest {
    method: String,
    req_headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct HttpResponse {
    status: i64,
    request: HttpRequest,
    resp_headers: HashMap<String, String>,
    text: Option<String>,
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

    pub async fn fetch_http_response(
        page: &Page,
        url: String,
        capture_headers: bool,
    ) -> Result<HttpResponse, AppError> {
        page.execute(network::EnableParams::default()).await?;

        let mut req_s = page
            .event_listener::<network::EventRequestWillBeSent>()
            .await?;
        let mut resp_s = page
            .event_listener::<network::EventResponseReceived>()
            .await?;
        let mut fin_s = page
            .event_listener::<network::EventLoadingFinished>()
            .await?;

        page.goto(url).await?;

        let mut doc_rid: Option<network::RequestId> = None;

        let mut method: Option<String> = None;
        let mut req_headers_raw: Option<network::Headers> = None;

        let mut status: Option<i64> = None;
        let mut resp_headers_raw: Option<network::Headers> = None;

        loop {
            tokio::select! {
                Some(ev) = req_s.next() => {
                    if ev.r#type != Some(network::ResourceType::Document) {
                        continue;
                    }

                    doc_rid = Some(ev.request_id.clone());
                    method = Some(ev.request.method.clone());

                    if !capture_headers {
                        req_headers_raw = Some(ev.request.headers.clone());
                    }
                }

                Some(ev) = resp_s.next() => {
                    if ev.r#type != network::ResourceType::Document {
                        continue;
                    }

                    doc_rid = Some(ev.request_id.clone());
                    status = Some(ev.response.status as i64);

                    if !capture_headers {
                        resp_headers_raw = Some(ev.response.headers.clone());
                    }
                }

                Some(ev) = fin_s.next() => {
                    let Some(rid) = doc_rid.as_ref() else { continue; };
                    if &ev.request_id != rid { continue; }

                    let Some(method) = method.clone() else { continue; };
                    let Some(status) = status else { continue; };

                    let rid = rid.clone();
                    let body = page
                        .execute(network::GetResponseBodyParams { request_id: rid })
                        .await?;

                    let body_str = &body.body;

                    let bytes: Vec<u8> = if body.base64_encoded {
                        general_purpose::STANDARD.decode(body_str.as_bytes())?
                    } else {
                        body_str.as_bytes().to_vec()
                    };

                    let text = String::from_utf8(bytes).ok();

                    let (req_headers, resp_headers) = if capture_headers {
                        (HashMap::new(), HashMap::new())
                    } else {
                        (
                            headers_to_string_map(req_headers_raw.as_ref()),
                            headers_to_string_map(resp_headers_raw.as_ref()),
                        )
                    };

                    return Ok(HttpResponse {
                        status,
                        request: HttpRequest {
                            method,
                            req_headers,
                        },
                        resp_headers,
                        text,
                    });
                }
            }
        }
    }
}

fn headers_to_string_map(h: Option<&network::Headers>) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Some(h) = h else {
        return out;
    };

    let v = serde_json::to_value(h).unwrap_or(serde_json::Value::Null);
    let Some(obj) = v.as_object() else {
        return out;
    };

    for (k, vv) in obj {
        let s = match vv {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => String::new(),
            _ => vv.to_string(),
        };
        out.insert(k.clone(), s);
    }

    out
}

#[async_trait]
impl<'a> Task for HeadlessBrowserFetcher<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        let tab = self.pool.get().await?;
        let page = tab.goto(message.uri).await?;
        let html = page.wait_for_navigation().await?.content().await?;

        unimplemented!()
    }
}
