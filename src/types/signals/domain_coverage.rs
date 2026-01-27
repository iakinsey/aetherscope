use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::types::traits::signal::Signal;

// Approximate coverage tracking per site (eTLD+1 or IP).
// Uses sketches to estimate discovered vs fetched URLs
// for crawl balancing and saturation detection.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainCoverage {
    // Hash of eTLD+1
    pub domain_key: Vec<u8>,
    // HyperLogLog sketch of discovered URLs
    pub hll_discovered: Vec<u8>,
    // HyperLogLog sketch of fetched URLs
    pub hll_fetched: Vec<u8>,
    // Most recent update timestamp
    pub last_update_ts: DateTime<Utc>,
}

impl Signal for DomainCoverage {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS domain_coverage (
            domain_key      blob PRIMARY KEY,
            hll_discovered  blob,
            hll_fetched     blob,
            last_update_ts  timestamp
        )
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO domain_coverage (
            domain_key, hll_discovered, hll_fetched, last_update_ts
        ) VALUES (?, ?, ?, ?)
    "#;

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.domain_key.clone(),
            self.hll_discovered.clone(),
            self.hll_fetched.clone(),
            self.last_update_ts.naive_utc()
        )
    }
}
