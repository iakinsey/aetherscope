use std::sync::Arc;

use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};
use xxhrs::XXH3_128;

use crate::{
    types::{
        error::AppError,
        structs::{record::Record, signal_base::SignalBase},
        traits::{object_store::ObjectStore, signal::Signal},
    },
    utils::cassandra::DbSession,
};

// Discovery depth metadata for a URL.
// Records how far a URL is from initial seeds and when it was first seen.
#[derive(Debug, Clone, PartialEq)]
pub struct UrlDepth {
    // Hash of the URL
    pub url_key: Vec<u8>,
    // Depth from seed
    pub depth: i32,
    // Timestamp when this URL was discovered
    pub discovered_ts: DateTime<Utc>,
}

impl Signal for UrlDepth {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS url_depth (
            url_key        blob PRIMARY KEY,
            depth          int,
            discovered_ts  timestamp
        )
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO url_depth (
            url_key, depth, discovered_ts
        ) VALUES (?, ?, ?)
    "#;

    async fn from_record(
        session: Arc<DbSession>,
        object_store: Arc<dyn ObjectStore>,
        base: SignalBase,
        record: Record,
    ) -> Result<Vec<Self>, AppError> {
        let url_key = XXH3_128::hash(record.uri.as_bytes()).to_be_bytes().to_vec();

        Ok(vec![Self {
            url_key,
            depth: record.depth,
            discovered_ts: record.discovered,
        }])
    }

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.url_key.clone(),
            self.depth,
            self.discovered_ts.naive_utc()
        )
    }
}
