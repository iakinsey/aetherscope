use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use crate::{
    types::{
        error::AppError,
        structs::{
            metadata::http_response::{HttpRequest, HttpResponse},
            record::{Record, RecordMetadata},
        },
        traits::{object_store::ObjectStore, task::Task},
    },
    utils::{dependencies::dependencies, fs::download_browser, sync::TabPool},
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chromiumoxide::{Browser, BrowserConfig, Page};
use chromiumoxide::{browser::HeadlessMode, cdp::browser_protocol::network};
use fastpool::bounded::{Object, Pool, PoolConfig};
use futures::StreamExt;
use tokio::{
    spawn,
    task::JoinHandle,
    time::{Instant, sleep_until},
};
use uuid::Uuid;

pub struct HeadlessBrowserConfig {
    http_proxy: Option<String>,
    browser_path: Option<String>,
    object_store: String,
    idle_timeout: i32,
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

        let pool = Pool::new(PoolConfig::new(16), TabPool::new(Arc::clone(&browser)));

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
        page: Object<TabPool>,
        url: String,
        object_store: Arc<dyn ObjectStore>,
        idle_timeout: Duration,
    ) -> Result<HttpResponse, AppError> {
        let mut reqs = page
            .event_listener::<network::EventRequestWillBeSent>()
            .await?;
        let mut resps = page
            .event_listener::<network::EventResponseReceived>()
            .await?;

        let mut nav = Box::pin(page.goto(url));
        let mut request_headers: Option<network::Headers> = None;
        let mut response_headers: Option<network::Headers> = None;
        let mut last_event = Instant::now();
        let mut status: Option<i64> = None;
        let mut body: Option<Vec<u8>> = None;
        let method = "GET";

        loop {
            tokio::select! {
                _ = &mut nav => {
                    break;
                }
                Some(e) = reqs.next() => {
                    last_event = Instant::now();
                    request_headers = Some(e.request.headers.clone());
                }
                Some(e) = resps.next() => {
                    last_event = Instant::now();
                    status = Some(e.response.status);
                    response_headers = Some(e.response.headers.clone());

                    let request_id = e.request_id.clone();
                    let resp = page
                        .execute(network::GetResponseBodyParams { request_id })
                        .await?;

                    body = Some(if resp.base64_encoded {
                        general_purpose::STANDARD.decode(&resp.body)?
                    } else {
                        resp.body.clone().into_bytes()
                    });
                }
                _ = sleep_until(last_event + idle_timeout) => {
                    break;
                }
            }
        }

        let request_headers = headers_to_hashmap(request_headers);
        let response_headers = headers_to_hashmap(response_headers);
        let mut key: Option<String> = None;

        if let Some(body) = body {
            let storage_key = Uuid::new_v4().to_string();
            object_store.put(&storage_key, &body).await?;
            key = Some(storage_key)
        }

        Ok(HttpResponse {
            request: HttpRequest {
                method: method.to_string(),
                request_headers,
            },
            response_headers,
            status,
            key,
            error: None,
        })
    }
}

pub fn headers_to_hashmap(headers: Option<network::Headers>) -> HashMap<String, String> {
    let mut out = HashMap::new();

    let Some(h) = headers else {
        return out;
    };

    let Some(obj) = h.inner().as_object() else {
        return out;
    };

    for (k, v) in obj {
        let val = v
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| v.to_string());
        out.insert(k.clone(), val);
    }

    out
}

#[async_trait]
impl<'a> Task for HeadlessBrowserFetcher<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        let tab = self.pool.get().await?;
        let response = match Self::fetch_http_response(
            tab,
            message.uri.clone(),
            self.object_store.clone(),
            Duration::from_secs(self.config.idle_timeout as u64),
        )
        .await
        {
            Ok(resp) => resp,
            Err(e) => HttpResponse {
                status: None,
                request: HttpRequest {
                    method: "GET".to_string(),
                    request_headers: HashMap::new(),
                },
                response_headers: HashMap::new(),
                key: None,
                error: Some(e.to_string()),
            },
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
            object_store: store_name.to_string(),
            idle_timeout: 30,
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
