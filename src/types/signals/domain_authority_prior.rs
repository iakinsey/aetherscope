use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::types::traits::signal::Signal;

// Static or slowly changing authority prior per site.
// Used to bootstrap importance before sufficient crawl data exists.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAuthorityPrior {
    // Hash of eTLD+1
    pub domain_key: Vec<u8>,
    // Authority prior score
    pub authority: f64,
    // Most recent update timestamp
    pub updated_ts: DateTime<Utc>,
}

impl Signal for DomainAuthorityPrior {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS domain_authority_prior (
            domain_key  blob PRIMARY KEY,
            authority   double,
            updated_ts  timestamp
        )
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO domain_authority_prior (
            domain_key, authority, updated_ts
        ) VALUES (?, ?, ?)
    "#;

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.domain_key.clone(),
            self.authority,
            self.updated_ts.naive_utc()
        )
    }
}
