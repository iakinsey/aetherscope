use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    types::{
        configs::tasks::url_extractor_config::UrlExtractorConfig,
        error::AppError,
        structs::{
            metadata::uris::Uris,
            record::{Record, RecordMetadata},
        },
        traits::{object_store::ObjectStore, task::Task},
    },
    utils::{dependencies::dependencies, fsm::UriExtractorFSM},
};

pub struct UrlExtractor<'a> {
    config: &'a UrlExtractorConfig,
    object_store: Arc<dyn ObjectStore>,
}

impl<'a> UrlExtractor<'a> {
    pub async fn new(config: &'a UrlExtractorConfig) -> Result<Self, AppError> {
        let object_store = dependencies()
            .lock()
            .await
            .get_object_store(&config.object_store)?;

        Ok(Self {
            config,
            object_store,
        })
    }
}

#[async_trait]
impl<'a> Task for UrlExtractor<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        let mut metadata = vec![];

        for meta in message.metadata {
            let http_response = match meta.clone() {
                RecordMetadata::HttpResponse(r) => r,
                _ => continue,
            };

            if let Some(key) = http_response.key {
                let buf = self.object_store.get_stream(&key).await?;
                let fsm = UriExtractorFSM::new(buf, message.uri.clone())?;
                let uris = fsm.perform().await?;

                metadata.push(RecordMetadata::Uris(Uris { uris }));
            }

            metadata.push(meta);
        }

        Ok(Record {
            uri: message.uri,
            task_id: message.task_id,
            metadata: metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, env::temp_dir};

    use chrono::Utc;
    use uuid::Uuid;

    use crate::{
        services::object_store::fs::FileSystemObjectStore,
        types::structs::metadata::http_response::{HttpRequest, HttpResponse},
    };

    use super::*;

    #[tokio::test]
    async fn test_url_extraction() {
        let path = temp_dir().join(Uuid::new_v4().to_string());
        let store = FileSystemObjectStore::new(path).await.unwrap();
        let store_name = "test-object-store";
        let task_id = Uuid::new_v4().to_string();
        let contents = r#"
            I am an html document.
            <a href="/test">Hello world</a>
            <a href="testme">Hello world</a>
            <a tag="h1234" href="testagain.com">Hello world</a>
        "#
        .as_bytes();

        let key = Uuid::new_v4().to_string();
        store.put(&key, contents).await.unwrap();

        let response = HttpResponse {
            status: Some(200),
            request: HttpRequest {
                method: "GET".to_string(),
                request_headers: HashMap::new(),
                timestamp: Utc::now(),
            },
            response_headers: HashMap::new(),
            key: Some(key),
            error: None,
            timestamp: None,
        };

        dependencies()
            .lock()
            .await
            .set_object_store(store_name, Arc::new(store))
            .unwrap();

        let config = &UrlExtractorConfig {
            object_store: store_name.to_string(),
        };
        let extractor = UrlExtractor::new(config).await.unwrap();
        let record = Record {
            uri: "http://example.com".to_string(),
            task_id: task_id,
            metadata: vec![RecordMetadata::HttpResponse(response)],
        };

        let response = extractor.on_message(record).await.unwrap();

        assert_eq!(response.metadata.len(), 2);

        for meta in response.metadata {
            let uris = match meta {
                RecordMetadata::Uris(u) => u,
                _ => continue,
            };

            assert_eq!(uris.uris.len(), 3);
        }
    }
}
