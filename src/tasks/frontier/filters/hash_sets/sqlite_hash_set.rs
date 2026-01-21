use std::iter::repeat;

use async_trait::async_trait;
use sqlx::{Pool, Row, Sqlite, SqlitePool, query};

use crate::types::{
    configs::unique_filter_config::SqliteHashSetConfig, error::AppError, traits::hash_set::HashSet,
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
impl HashSet for SqliteHashSet {
    async fn contains_entities(
        &self,
        entities: Vec<String>,
    ) -> Result<Vec<(String, bool)>, AppError> {
        let insert_vals = repeat("(?)")
            .take(entities.len())
            .collect::<Vec<_>>()
            .join(",");

        let sql = format!(
            r#"
        WITH input(name) AS (
            VALUES {insert_vals}
        ),
        inserted AS (
            INSERT INTO entity(name)
            SELECT name FROM input
            ON CONFLICT(name) DO NOTHING
            RETURNING name
        )
        SELECT
            i.name,
            (ins.name IS NOT NULL) AS inserted
        FROM input i
        LEFT JOIN inserted ins USING (name);
        "#
        );

        let mut q = query(&sql);

        for e in entities {
            q = q.bind(e);
        }

        let rows = q.fetch_all(&self.db).await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("name"), r.get::<i64, _>("inserted") != 0))
            .collect())
    }
}
