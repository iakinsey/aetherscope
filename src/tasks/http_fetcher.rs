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
use reqwest::Client;
use uuid::Uuid;
pub struct HttpFetcher<'a> {
    config: &'a HttpFetcherConfig,
    client: Client,
    object_store: Arc<dyn ObjectStore>,
}

impl<'a> HttpFetcher<'a> {
    pub async fn new(config: &'a HttpFetcherConfig) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout as u64))
            .user_agent(get_user_agent(config.user_agent.clone()))
            .build()?;

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
mod tests {}
