use std::collections::HashSet;

use async_trait::async_trait;
use sqlx::{Pool, Row, Sqlite, SqlitePool, query};

use crate::types::{
    configs::filters::unique_filter_config::SqliteHashSetConfig, error::AppError,
    traits::check_hash_set::CheckHashSet,
};

pub struct SqliteHashSet {
    db: SqlitePool,
}

impl SqliteHashSet {
    pub async fn new(config: SqliteHashSetConfig) -> Result<Self, AppError> {
        let db = SqlitePool::connect(&config.path).await?;

        Self::init_db(&db).await?;

        Ok(Self { db })
    }

    pub async fn init_db(db: &Pool<Sqlite>) -> Result<(), AppError> {
        sqlx::query("CREATE TABLE entity (name TEXT PRIMARY KEY)")
            .execute(db)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl CheckHashSet for SqliteHashSet {
    async fn contains_entities(
        &self,
        entities: Vec<String>,
    ) -> Result<Vec<(String, bool)>, AppError> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let vals = std::iter::repeat("(?)")
            .take(entities.len())
            .collect::<Vec<_>>()
            .join(",");

        let select_sql = format!(
            r#"
        WITH input(name) AS (VALUES {vals})
        SELECT e.name
        FROM entity e
        JOIN input i ON i.name = e.name;
        "#
        );

        let insert_sql = format!(
            r#"
        WITH input(name) AS (VALUES {vals})
        INSERT OR IGNORE INTO entity(name)
        SELECT name FROM input;
        "#
        );

        let mut tx = self.db.begin().await?;
        let mut sel = query(&select_sql);

        for e in &entities {
            sel = sel.bind(e);
        }

        let existing_rows = sel.fetch_all(&mut *tx).await?;
        let existing: HashSet<String> = existing_rows
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect();
        let mut ins = query(&insert_sql);

        for e in &entities {
            ins = ins.bind(e);
        }

        ins.execute(&mut *tx).await?;
        tx.commit().await?;

        Ok(entities
            .into_iter()
            .map(|name| {
                let existed = existing.contains(name.as_str());
                (name, existed)
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn test_contains_entities() {
        let config = SqliteHashSetConfig {
            path: ":memory:".to_string(),
        };

        let hash_set = SqliteHashSet::new(config).await.unwrap();
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
