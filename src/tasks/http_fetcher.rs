use std::{collections::HashMap, time::Duration};

use crate::{
    types::{
        configs::http_fetcher_config::HttpFetcherConfig, error::AppError, structs::record::Record,
        traits::task::Task,
    },
    utils::web::get_user_agent,
};
use async_trait::async_trait;
use reqwest::Client;

pub struct HttpFetcher<'a> {
    config: &'a HttpFetcherConfig,
    client: Client,
}

impl<'a> HttpFetcher<'a> {
    pub async fn new(config: &'a HttpFetcherConfig) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout as u64))
            .user_agent(get_user_agent(config.user_agent.clone()))
            .build()?;

        Ok(Self { config, client })
    }
}

#[async_trait]
impl<'a> Task for HttpFetcher<'a> {
    async fn on_message(&self, message: Record) -> Result<Record, AppError> {
        let resp = self.client.get(message.uri).send().await?;
        let status = resp.status().as_u16();
        let headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_string(),
                    v.to_str().unwrap_or("<non-utf8>").to_string(),
                )
            })
            .collect();

        unimplemented!();
    }
}

#[cfg(test)]
mod tests {}
