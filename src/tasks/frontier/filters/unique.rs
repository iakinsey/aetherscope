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
    bloom_filter: Option<BloomFilter>,
    hash_set: Option<Box<dyn CheckHashSet>>,
}

impl UniqueFilter {
    pub async fn new(config: UniqueFilterConfig) -> Result<Self, AppError> {
        let bloom_filter = match config.bloom_filter.enable {
            true => Some(
                BloomFilter::with_false_pos(config.bloom_filter.false_positive_rate)
                    .expected_items(config.bloom_filter.expected_size),
            ),
            false => None,
        };

        let hash_set = Self::get_hash_set(config.hash_set).await?;

        Ok(Self {
            bloom_filter,
            hash_set,
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
        Ok(match (&mut self.bloom_filter, &mut self.hash_set) {
            (Some(bloom_filter), None) => uris
                .iter()
                .map(|u| (u.clone(), bloom_filter.contains(u)))
                .collect(),
            (None, Some(hash_set)) => hash_set.contains_entities(uris).await?,
            (Some(bloom_filter), Some(hash_set)) => {
                let (results, false_positives): (Vec<_>, Vec<_>) =
                    uris.into_iter().partition(|u| bloom_filter.contains(u));

                let mut results: Vec<(String, bool)> =
                    results.into_iter().map(|u| (u, false)).collect();

                results.extend(hash_set.contains_entities(false_positives).await?);

                results
            }
            (None, None) => uris.iter().map(|u| (u.clone(), true)).collect(),
        })
    }
}
