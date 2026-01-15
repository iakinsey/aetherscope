use std::time::Duration;

use reqwest::{Client, Proxy};
use robotxt::Robots;
use url::Url;

use crate::{
    types::{
        configs::robots_filter_config::RobotsFilterConfig, error::AppError,
        traits::frontier_filter::FrontierFilter,
    },
    utils::web::{fetch_http_simple, get_robots_url, get_user_agent},
};

pub struct RobotsFilter {
    client: Client,
    user_agent: String,
}

impl RobotsFilter {
    pub fn new(robots_filter_config: RobotsFilterConfig) -> Result<Self, AppError> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(
                robots_filter_config.http_config.timeout as u64,
            ))
            .user_agent(get_user_agent(
                robots_filter_config.http_config.user_agent.clone(),
            ));

        if let Some(proxy_server) = &robots_filter_config.http_config.proxy_server {
            builder = builder.proxy(Proxy::all(proxy_server)?);
        }

        let client = builder.build()?;
        let user_agent = get_user_agent(robots_filter_config.http_config.user_agent);

        Ok(Self { client, user_agent })
    }
}

impl FrontierFilter for RobotsFilter {
    async fn filter(
        self,
        uris: Vec<String>,
        origin: &str,
    ) -> Result<Vec<(String, bool)>, AppError> {
        let robots_url = get_robots_url(origin)?;
        let contents = fetch_http_simple(self.client, &robots_url).await?;
        let robots = Robots::from_bytes(contents.as_ref(), &self.user_agent);
        let mut results = vec![];

        for uri in uris {
            let url = Url::parse(&uri)?;
            let allowed = robots.is_absolute_allowed(&url);

            results.push((uri, allowed))
        }

        results
    }
}
