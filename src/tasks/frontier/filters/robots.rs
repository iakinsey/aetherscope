use std::{collections::HashMap, time::Duration};

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
    async fn perform(
        self,
        uris: Vec<String>,
        _origin: &str,
    ) -> Result<Vec<(String, bool)>, AppError> {
        let mut buckets: HashMap<(String, String, Option<u16>), Vec<String>> = HashMap::new();

        for uri in uris {
            let url = Url::parse(&uri)?;
            let host = url
                .host_str()
                .ok_or_else(|| AppError::from("URL missing host"))?
                .to_owned();

            let scheme = url.scheme().to_owned();
            let port = url.port();

            buckets.entry((scheme, host, port)).or_default().push(uri);
        }

        let mut results = Vec::new();

        for ((scheme, host, port), bucket) in buckets {
            let robots_origin = match port {
                Some(p) => format!("{scheme}://{host}:{p}"),
                None => format!("{scheme}://{host}"),
            };

            let robots_url = get_robots_url(&robots_origin)?;
            let contents = fetch_http_simple(self.client.clone(), &robots_url).await?;
            let robots = Robots::from_bytes(contents.as_ref(), &self.user_agent);

            for uri in bucket {
                let url = Url::parse(&uri)?;
                let allowed = robots.is_relative_allowed(url.path());
                results.push((uri, allowed));
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use httpmock::{Method::GET, MockServer};

    use crate::{
        tasks::frontier::filters::robots::RobotsFilter,
        types::{
            configs::{
                http_fetcher_config::BasicHttpFetcherConfig,
                robots_filter_config::RobotsFilterConfig,
            },
            traits::frontier_filter::FrontierFilter,
        },
    };

    #[tokio::test]
    async fn test_filter_success() {
        let user_agent = "test-user-agent";
        let config = RobotsFilterConfig {
            http_config: BasicHttpFetcherConfig {
                proxy_server: None,
                timeout: 32,
                user_agent: Some(user_agent.to_string()),
            },
        };
        let filter = RobotsFilter::new(config).unwrap();
        let robotstxt = format!(
            r#"User-agent: {}
Disallow: /admin/
Disallow: /example/"#,
            user_agent
        );

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .header("user-agent", user_agent)
                .path("/robots.txt");
            then.status(200).body(robotstxt);
        });
        let base = server.base_url();
        let mut map = HashMap::new();

        map.insert(format!("{}/allowed", base), true);
        map.insert(format!("{}/admin/", base), false);
        map.insert(format!("{}/example/", base), false);
        map.insert(format!("{}/admin/no", base), false);
        map.insert(format!("{}/example/no", base), false);

        let uris = map.keys().cloned().collect();
        let filters = filter.perform(uris, "").await.unwrap();

        mock.assert();

        let mut result_map: HashMap<String, bool> = filters.into_iter().collect();

        assert_eq!(result_map.len(), map.len());

        for (uri, expected) in map {
            let actual = result_map.remove(&uri).expect("missing uri in result");
            assert_eq!(actual, expected, "mismatch for uri {}", uri);
        }
    }

    #[tokio::test]
    async fn test_filter_invalid_response() {}

    #[tokio::test]
    async fn test_filter_no_response() {
        unimplemented!()
    }
}
