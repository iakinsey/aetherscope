use std::sync::Arc;

use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::types::{
    error::AppError,
    structs::record::Record,
    traits::signal::{DbSession, Signal},
};

// Statistics for URL path prefixes or templates within a host.
// Used to detect low-yield, duplicate-heavy, or spammy patterns
// and adjust crawl priority accordingly.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefixStats {
    // Hash of the host (scheme+host+port)
    pub host_key: Vec<u8>,
    // Hash of the prefix/template id
    pub prefix_key: Vec<u8>,
    // Most recent update timestamp
    pub last_update_ts: DateTime<Utc>,
    // EMA of duplicate pages
    pub dup_page_ema: f64,
    // EMA of novelty
    pub novelty_ema: f64,
    // EMA of near-duplicate rate
    pub near_dup_ema: f64,
    // EMA of content variance
    pub variance_ema: f64,
}

impl Signal for PrefixStats {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS prefix_stats (
            host_key        blob,
            prefix_key      blob,
            last_update_ts  timestamp,
            dup_page_ema    double,
            novelty_ema     double,
            near_dup_ema    double,
            variance_ema    double,
            PRIMARY KEY ((host_key), prefix_key)
        )
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO prefix_stats (
            host_key, prefix_key,
            last_update_ts,
            dup_page_ema, novelty_ema, near_dup_ema, variance_ema
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
    "#;

    fn from_record(session: Arc<DbSession>, record: Record) -> Result<Vec<Self>, AppError> {
        unimplemented!()
    }

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.host_key.clone(),
            self.prefix_key.clone(),
            self.last_update_ts.naive_utc(),
            self.dup_page_ema,
            self.novelty_ema,
            self.near_dup_ema,
            self.variance_ema
        )
    }
}
