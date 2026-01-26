use async_trait::async_trait;
use redis::{Client, cmd, pipe};

use crate::types::{
    configs::filters::unique_filter_config::RedisHashSetConfig, error::AppError,
    traits::check_hash_set::CheckHashSet,
};
use redis::aio::ConnectionManager;

pub struct RedisHashSet {
    conn: ConnectionManager,
}

impl RedisHashSet {
    pub async fn new(config: RedisHashSetConfig) -> Result<Self, AppError> {
        let client = Client::open(config.uri)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self { conn })
    }
}

#[async_trait]
impl CheckHashSet for RedisHashSet {
    async fn contains_entities(
        &self,
        entities: Vec<String>,
    ) -> Result<Vec<(String, bool)>, AppError> {
        let mut conn = self.conn.clone();

        let vals: Vec<Option<u8>> = cmd("MGET")
            .arg(&entities)
            .query_async(&mut conn)
            .await
            .map_err(AppError::from)?;

        let existed: Vec<bool> = vals.iter().map(|v| v.is_some()).collect();
        let mut pipe = pipe();

        for (k, did_exist) in entities.iter().zip(existed.iter()) {
            if !did_exist {
                pipe.cmd("SETNX").arg(k).arg(1u8);
            }
        }

        if !pipe.is_empty() {
            let _: Vec<i32> = pipe.query_async(&mut conn).await.map_err(AppError::from)?;
        }

        Ok(entities.into_iter().zip(existed.into_iter()).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn test_contains_entities() {
        let config = RedisHashSetConfig {
            uri: "redis://localhost:6379".to_string(),
        };

        let hash_set = RedisHashSet::new(config).await.unwrap();
        let entities: Vec<String> = (0..100).map(|_| Uuid::new_v4().to_string()).collect();
        let results = hash_set.contains_entities(entities.clone()).await.unwrap();

        assert!(results.iter().all(|(_, b)| !*b));

        let mut some_true: Vec<String> = (101..151).map(|_| Uuid::new_v4().to_string()).collect();

        some_true.extend(entities);

        let results = hash_set.contains_entities(some_true).await.unwrap();

        let mut counts = HashMap::new();
        for (_, b) in &results {
            *counts.entry(*b).or_insert(0usize) += 1;
        }

        assert_eq!(counts.get(&true), Some(&100));
        assert_eq!(counts.get(&false), Some(&50));
    }
}
