use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    types::{
        error::AppError,
        structs::{
            metadata::http_response::{HttpRequest, HttpResponse},
            record::{self, Record, RecordMetadata},
        },
        traits::{object_store::ObjectStore, task::Task},
    },
    utils::{dependencies::dependencies, fs::download_browser, sync::TabPool},
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chromiumoxide::{Browser, BrowserConfig, Page};
use chromiumoxide::{browser::HeadlessMode, cdp::browser_protocol::network};
use fastpool::bounded::{Pool, PoolConfig};
use futures::StreamExt;
use tokio::{spawn, task::JoinHandle};
use uuid::Uuid;

pub struct HeadlessBrowserConfig {
    http_proxy: Option<String>,
    browser_path: Option<String>,
    enable_http_metrics: bool,
    object_store: String,
}

pub struct HeadlessBrowserFetcher<'a> {
    browser: Arc<Browser>,
    _handle: JoinHandle<()>,
    pool: Arc<Pool<TabPool>>,
    config: &'a HeadlessBrowserConfig,
    object_store: Arc<dyn ObjectStore>,
}

impl<'a> HeadlessBrowserFetcher<'a> {
    pub async fn new(config: &'a HeadlessBrowserConfig) -> Result<Self, AppError> {
        let browser_path = match &config.browser_path {
            Some(p) => PathBuf::from(p),
            None => download_browser(None).await?,
        };

        let mut browser_config = BrowserConfig::builder()
            .headless_mode(HeadlessMode::True)
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

        let object_store = dependencies()
            .lock()
            .await
            .get_object_store(&config.object_store)?;

        Ok(Self {
            browser,
            _handle,
            pool,
            config,
            object_store,
        })
    }

    pub async fn fetch_http_response(
        page: &Page,
        url: String,
        capture_headers: bool,
        object_store: Arc<dyn ObjectStore>,
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
        let object_store = object_store;

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
                        .await
                        .map_err(|e| AppError::Http {
                            status: status,
                            method: method.clone(),
                            message: e.to_string(),
                        })?;

                    let body_str = &body.body;

                    let bytes: Vec<u8> = if body.base64_encoded {
                        general_purpose::STANDARD
                            .decode(body_str.as_bytes())
                            .map_err(|e| AppError::Http {
                                status: status,
                                method: method.clone(),
                                message: e.to_string(),
                        })?
                    } else {
                        body_str.as_bytes().to_vec()
                    };

                    let key = Uuid::new_v4().to_string();

                    object_store.put(&key, bytes.as_slice()).await.map_err(|e| AppError::Http {
                            status: status,
                            method: method.clone(),
                            message: e.to_string(),
                        })?;

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
                        key: Some(key),
                        error: None,
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
        let page = tab.goto(message.uri.clone()).await?;
        let response = match Self::fetch_http_response(
            page,
            message.uri.clone(),
            self.config.enable_http_metrics,
            self.object_store.clone(),
        )
        .await
        {
            Ok(resp) => resp,
            Err(AppError::Http {
                status,
                method,
                message,
            }) => HttpResponse {
                status,
                request: HttpRequest {
                    method,
                    req_headers: HashMap::new(),
                },
                resp_headers: HashMap::new(),
                key: None,
                error: Some(message),
            },
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Record {
            uri: message.uri,
            task_id: message.task_id,
            metadata: vec![RecordMetadata::HttpResponse(response)],
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use httpmock::{Method::GET, MockServer};

    use crate::services::object_store::fs::FileSystemObjectStore;

    use super::*;

    #[tokio::test]
    async fn test_request_success() {
        let path = temp_dir().join(Uuid::new_v4().to_string());
        let store = FileSystemObjectStore::new(path).await.unwrap();
        let store_name = "test-object-store";
        let task_id = Uuid::new_v4().to_string();
        let test_response = "This is a test response.";

        dependencies()
            .lock()
            .await
            .set_object_store(store_name, Arc::new(store))
            .unwrap();

        let config = HeadlessBrowserConfig {
            http_proxy: None,
            browser_path: None,
            enable_http_metrics: true,
            object_store: store_name.to_string(),
        };

        let fetcher = HeadlessBrowserFetcher::new(&config).await.unwrap();

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/");

            then.status(200).body(test_response);
        });

        let record = Record {
            uri: server.base_url(),
            task_id: task_id,
            metadata: vec![],
        };

        let response = fetcher.on_message(record).await.unwrap();

        mock.assert();

        let http_response: &HttpResponse = match response.metadata.first() {
            Some(RecordMetadata::HttpResponse(r)) => r,
            _ => panic!("headless browser did not create a response object"),
        };

        let key = http_response.key.clone().unwrap();

        let object_store = dependencies()
            .lock()
            .await
            .get_object_store(&config.object_store)
            .unwrap();

        let response_body = object_store.get(&key).await.unwrap();
        let response_string = String::from_utf8(response_body).unwrap();

        assert_eq!(response_string, test_response)
    }

    #[tokio::test]
    async fn test_request_success_no_metrics() {
        unimplemented!();
    }

    #[tokio::test]
    async fn test_request_success_http_proxy() {
        unimplemented!();
    }

    #[tokio::test]
    async fn test_request_bad_response() {
        unimplemented!();
    }

    #[tokio::test]
    async fn test_request_error() {
        unimplemented!();
    }
}
