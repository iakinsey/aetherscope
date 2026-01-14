use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    types::{
        configs::url_extractor_config::UrlExtractorConfig,
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
