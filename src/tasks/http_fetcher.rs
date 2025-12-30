use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    types::{
        configs::http_fetcher_config::HttpFetcherConfig,
        error::AppError,
        structs::{
            metadata::http_response::{HttpRequest, HttpResponse},
            record::{Record, RecordMetadata},
        },
        traits::{object_store::ObjectStore, task::Task},
    },
    utils::{dependencies::dependencies, web::get_user_agent},
};
use async_trait::async_trait;
use futures::StreamExt;
use futures_util::TryStreamExt;
use reqwest::{Client, Proxy};
use uuid::Uuid;
pub struct HttpFetcher<'a> {
    config: &'a HttpFetcherConfig,
    client: Client,
    object_store: Arc<dyn ObjectStore>,
}

impl<'a> HttpFetcher<'a> {
    pub async fn new(config: &'a HttpFetcherConfig) -> Result<Self, AppError> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout as u64))
            .user_agent(get_user_agent(config.user_agent.clone()));

        if let Some(proxy_server) = &config.proxy_server {
            builder = builder.proxy(Proxy::all(proxy_server)?);
        }

        let client = builder.build()?;

        let object_store = dependencies()
            .lock()
            .await
            .get_object_store(&config.object_store)?;

        Ok(Self {
            config,
            client,
            object_store,
        })
    }

    pub async fn fetch_http_response(&self, uri: &str) -> Result<HttpResponse, AppError> {
        let req = self.client.get(uri).build()?;
        let (req_headers, method) = {
            let headers: HashMap<String, String> = req
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            let method: String = req.method().as_str().to_string();

            (headers, method)
        };
        let resp = self.client.execute(req).await?;
        let status = resp.status().as_u16();
        let response_headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let stream = resp.bytes_stream().map_err(AppError::from).boxed();
        let key = Uuid::new_v4().to_string();

        self.object_store.put_stream(&key, stream).await?;

        Ok(HttpResponse {
            request: HttpRequest {
                method: method.to_string(),
                request_headers: req_headers,
            },
            response_headers,
            status: Some(status as i64),
            key: Some(key),
            error: None,
        })
    }
}

#[async_trait]
impl<'a> Task for HttpFetcher<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        let response = match self.fetch_http_response(&message.uri).await {
            Ok(r) => r,
            Err(e) => HttpResponse {
                request: HttpRequest {
                    method: "GET".to_string(),
                    request_headers: HashMap::new(),
                },
                status: None,
                response_headers: HashMap::new(),
                key: None,
                error: Some(e.to_string()),
            },
        };
        let mut metadata = message.metadata;
        metadata.push(RecordMetadata::HttpResponse(response));

        Ok(Record {
            uri: message.uri,
            task_id: message.task_id,
            metadata: metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use httpmock::{Method::GET, MockServer};

    use crate::{services::object_store::fs::FileSystemObjectStore, utils::web::get_user_agent};

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

        let config = HttpFetcherConfig {
            user_agent: None,
            proxy_server: None,
            object_store: store_name.to_string(),
            timeout: 30,
        };

        let fetcher = HttpFetcher::new(&config).await.unwrap();
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/test")
                .header("user-agent", get_user_agent(config.user_agent.clone()));
            then.status(200).body(test_response);
        });

        let record = Record {
            uri: format!("{}/test", server.base_url()),
            task_id: task_id,
            metadata: vec![],
        };

        let response = fetcher.on_message(record).await.unwrap();

        mock.assert();

        let http_response: &HttpResponse = match response.metadata.first() {
            Some(RecordMetadata::HttpResponse(r)) => r,
            _ => panic!("headless browser did not create a response object"),
        };

        assert_eq!(http_response.status, Some(200));
        assert_eq!(http_response.request.method, "GET");
        assert_eq!(http_response.error, None);
        assert!(http_response.response_headers.len() > 1);
        assert!(http_response.request.request_headers.len() == 0);

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
    async fn test_request_bad_response() {
        let path = temp_dir().join(Uuid::new_v4().to_string());
        let store = FileSystemObjectStore::new(path).await.unwrap();
        let store_name = "test-object-store";
        let task_id = Uuid::new_v4().to_string();
        let test_response = "There was an error.";

        dependencies()
            .lock()
            .await
            .set_object_store(store_name, Arc::new(store))
            .unwrap();

        let config = HttpFetcherConfig {
            user_agent: None,
            proxy_server: None,
            object_store: store_name.to_string(),
            timeout: 30,
        };

        let fetcher = HttpFetcher::new(&config).await.unwrap();
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/")
                .header("user-agent", get_user_agent(config.user_agent.clone()));

            then.status(500).body(test_response);
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

        assert_eq!(http_response.status, Some(500));
        assert_eq!(http_response.request.method, "GET");
        assert_eq!(http_response.error, None);
        assert!(http_response.response_headers.len() > 1);
        assert!(http_response.request.request_headers.len() == 0);

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
    async fn test_request_error() {
        let path = temp_dir().join(Uuid::new_v4().to_string());
        let store = FileSystemObjectStore::new(path).await.unwrap();
        let store_name = "test-object-store";
        let task_id = Uuid::new_v4().to_string();

        dependencies()
            .lock()
            .await
            .set_object_store(store_name, Arc::new(store))
            .unwrap();

        let config = HttpFetcherConfig {
            user_agent: None,
            proxy_server: None,
            object_store: store_name.to_string(),
            timeout: 30,
        };

        let fetcher = HttpFetcher::new(&config).await.unwrap();
        let record = Record {
            uri: "http://127.0.0.1:9".to_string(),
            task_id: task_id,
            metadata: vec![],
        };

        let response = fetcher.on_message(record).await.unwrap();

        let http_response: &HttpResponse = match response.metadata.first() {
            Some(RecordMetadata::HttpResponse(r)) => r,
            _ => panic!("headless browser did not create a response object"),
        };

        assert!(
            http_response
                .error
                .clone()
                .unwrap()
                .contains("error sending request")
        );
    }
}
