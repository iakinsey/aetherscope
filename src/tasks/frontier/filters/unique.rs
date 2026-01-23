use std::collections::HashMap;

use fastbloom::BloomFilter;

use crate::{
    tasks::frontier::filters::hash_sets::{
        redis_hash_set::RedisHashSet, sqlite_hash_set::SqliteHashSet,
    },
    types::{
        configs::unique_filter_config::{HashSetConfig, UniqueFilterConfig},
        error::AppError,
        traits::{check_hash_set::CheckHashSet, frontier_filter::FrontierFilter},
    },
};

pub struct UniqueFilter {
    domain_bloom_filter: Option<BloomFilter>,
    url_bloom_filter: Option<BloomFilter>,
    domain_hash_set: Option<Box<dyn CheckHashSet>>,
    url_hash_set: Option<Box<dyn CheckHashSet>>,
}

impl UniqueFilter {
    pub async fn new(config: UniqueFilterConfig) -> Result<Self, AppError> {
        let domain_bloom_config = config.filter_domains.bloom_filter;
        let url_bloom_config = config.filter_urls.bloom_filter;

        let domain_bloom_filter = match domain_bloom_config.enable {
            true => Some(
                BloomFilter::with_false_pos(domain_bloom_config.false_positive_rate)
                    .expected_items(domain_bloom_config.expected_size),
            ),
            false => None,
        };

        let url_bloom_filter = match url_bloom_config.enable {
            true => Some(
                BloomFilter::with_false_pos(domain_bloom_config.false_positive_rate)
                    .expected_items(domain_bloom_config.expected_size),
            ),
            false => None,
        };

        let domain_hash_set = Self::get_hash_set(config.filter_domains.hash_set).await?;
        let url_hash_set = Self::get_hash_set(config.filter_urls.hash_set).await?;

        Ok(Self {
            domain_bloom_filter,
            url_bloom_filter,
            domain_hash_set,
            url_hash_set,
        })
    }

    pub async fn get_hash_set(
        config: HashSetConfig,
    ) -> Result<Option<Box<dyn CheckHashSet>>, AppError> {
        Ok(match config {
            HashSetConfig::Sqlite(conf) => {
                Some(Box::new(SqliteHashSet::new(conf.clone()).await?) as Box<dyn CheckHashSet>)
            }
            HashSetConfig::Redis(conf) => {
                Some(Box::new(RedisHashSet::new(conf.clone()).await?) as Box<dyn CheckHashSet>)
            }
            HashSetConfig::Empty => None,
        })
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
