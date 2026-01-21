use std::collections::HashMap;

use fastbloom::BloomFilter;

use crate::types::{
    configs::unique_filter_config::{self, UniqueFilterConfig},
    error::AppError,
    traits::frontier_filter::FrontierFilter,
};

pub struct UniqueFilter {
    bloom_filters: HashMap<&'static str, BloomFilter>,
}

const DOMAIN_KEY: &'static str = "domain";
const URL_KEY: &'static str = "url";

impl UniqueFilter {
    pub fn new(config: UniqueFilterConfig) -> Result<Self, AppError> {
        let mut bloom_filters = HashMap::new();
        let domain_bloom_config = config.filter_domains.bloom_filter;
        let url_bloom_config = config.filter_urls.bloom_filter;

        if domain_bloom_config.enable {
            let filter = BloomFilter::with_false_pos(domain_bloom_config.false_positive_rate)
                .expected_items(domain_bloom_config.expected_size);
            bloom_filters.insert(DOMAIN_KEY, filter);
        }

        if url_bloom_config.enable {
            let filter = BloomFilter::with_false_pos(domain_bloom_config.false_positive_rate)
                .expected_items(domain_bloom_config.expected_size);
            bloom_filters.insert(URL_KEY, filter);
        }

        Ok(Self { bloom_filters })
    }

    pub fn check_bloom(mut filter: BloomFilter, entities: Vec<String>) -> Vec<(String, bool)> {
        let mut results = vec![];

        for entity in entities {
            results.push((
                entity.clone(),
                match filter.contains(&entity) {
                    true => true,
                    false => {
                        filter.insert(&entity);
                        false
                    }
                },
            ));
        }

        results
    }
}

impl FrontierFilter for UniqueFilter {
    async fn perform(
        mut self,
        uris: Vec<String>,
        _origin: &str,
    ) -> Result<Vec<(String, bool)>, AppError> {
        unimplemented!()
    }
}
