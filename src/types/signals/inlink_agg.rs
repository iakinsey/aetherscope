use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::types::traits::signal::Signal;

// Aggregated inlink-based importance signals.
// Stores EMA-style authority for URLs, hosts, or sites,
// keyed by target and kind.
#[derive(Debug, Clone, PartialEq)]
pub struct InlinkAgg {
    // Hash of target entity (url_key OR host_key OR domain_key)
    pub target_key: Vec<u8>,
    // Kind of target (0=url,1=host,2=domain)
    pub kind: i8,
    // EMA of inlink count
    pub inlinks_ema: f64,
    // EMA of weighted inlink count
    pub w_inlinks_ema: f64,
    // Most recent update timestamp
    pub last_update_ts: DateTime<Utc>,
}

impl Signal for InlinkAgg {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS inlink_agg (
            target_key     blob,
            kind           tinyint,
            inlinks_ema    double,
            w_inlinks_ema  double,
            last_update_ts timestamp,
            PRIMARY KEY ((target_key), kind)
        )
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO inlink_agg (
            target_key, kind,
            inlinks_ema, w_inlinks_ema,
            last_update_ts
        ) VALUES (?, ?, ?, ?, ?)
    "#;

    fn from_record(
        record: crate::types::structs::record::Record,
    ) -> Result<Self, crate::types::error::AppError> {
        unimplemented!()
    }

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.target_key.clone(),
            self.kind,
            self.inlinks_ema,
            self.w_inlinks_ema,
            self.last_update_ts.naive_utc()
        )
    }
}
