use std::sync::Arc;

use bincode::{deserialize, serialize};
use cdrs_tokio::types::IntoRustByName;
use cdrs_tokio::types::blob::Blob;
use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};
use probabilistic_collections::hyperloglog::HyperLogLog;

use crate::types::structs::record::RecordMetadata;
use crate::types::{
    error::AppError,
    structs::{record::Record, signal_base::SignalBase},
    traits::{
        object_store::ObjectStore,
        signal::{DbSession, Signal},
    },
};

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

impl DomainCoverage {
    async fn get_latest(session: Arc<DbSession>, domain_key: Vec<u8>) -> Result<Self, AppError> {
        // If empty, emas are  0 and last_update_ts is now (probably can be passed in)
        const Q: &str = r#"
            SELECT
                hll_discovered,
                hll_fetched,
                last_update_ts
            FROM domain_coverage
            WHERE domain_key = ? 
        "#;

        let prepared = session.prepare(Q).await?;
        let result = session
            .exec_with_values(&prepared, query_values!(domain_key.clone()))
            .await?;

        let row = match result.response_body()?.into_rows() {
            Some(mut rows) if !rows.is_empty() => rows.remove(0),
            _ => {
                let hll = HyperLogLog::<String>::new(0.0325); // p = 10
                let bytes: Vec<u8> = serialize(&hll)?;
                return Ok(Self {
                    domain_key,
                    hll_discovered: bytes.clone(),
                    hll_fetched: bytes,
                    last_update_ts: Utc::now(),
                });
            }
        };

        let hll_discovered: Blob = row.get_r_by_name("hll_discovered")?;
        let hll_discovered: Vec<u8> = hll_discovered.into_vec();
        let hll_fetched: Blob = row.get_r_by_name("hll_fetched")?;
        let hll_fetched: Vec<u8> = hll_fetched.into_vec();
        let last_update_ts: Option<DateTime<Utc>> = row.get_by_name("last_update_ts")?;

        Ok(Self {
            domain_key,
            hll_discovered,
            hll_fetched,
            last_update_ts: last_update_ts.unwrap_or(Utc::now()),
        })
    }
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

    async fn from_record(
        session: Arc<DbSession>,
        _object_store: Arc<dyn ObjectStore>,
        base: SignalBase,
        record: Record,
    ) -> Result<Vec<Self>, AppError> {
        let domain_key = base.site_key;
        let latest = Self::get_latest(session, domain_key.clone()).await?;
        let mut hll_discovered: HyperLogLog<String> = deserialize(&latest.hll_discovered)?;
        let mut hll_fetched: HyperLogLog<String> = deserialize(&latest.hll_fetched)?;
        let last_update_ts = Utc::now();

        for m in record.metadata {
            if let RecordMetadata::HttpResponse(_) = m {
                hll_fetched.insert(&record.uri);
            };

            if let RecordMetadata::Uris(uris) = m {
                for uri in uris.uris {
                    hll_discovered.insert(&uri);
                }
            }
        }

        Ok(vec![Self {
            domain_key,
            hll_discovered: serialize(&hll_discovered)?,
            hll_fetched: serialize(&hll_fetched)?,
            last_update_ts,
        }])
    }

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.domain_key.clone(),
            self.hll_discovered.clone(),
            self.hll_fetched.clone(),
            self.last_update_ts.naive_utc()
        )
    }
}
