use std::str::FromStr;
use std::sync::Arc;

use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};
use url::Url;
use xxhrs::XXH3_128;

use crate::{
    types::{
        error::AppError,
        structs::{
            record::{Record, RecordMetadata},
            signal_base::SignalBase,
        },
        traits::{
            object_store::ObjectStore,
            signal::{DbSession, Signal},
        },
    },
    utils::{
        hash::jaccard_index,
        web::{extract_host, normalize_prefix},
    },
};

fn update_dup_page_ema(
    prev_ema: f64,
    prev_ts: DateTime<Utc>,
    now_ts: DateTime<Utc>,
    is_dup: bool,
    tau_seconds: f64,
) -> f64 {
    let dt = (now_ts - prev_ts).num_seconds().max(0) as f64;
    let decay = (-dt / tau_seconds).exp();

    let x = if is_dup { 1.0 } else { 0.0 };

    prev_ema * decay + x * (1.0 - decay)
}

// Statistics for URL path prefixes or templates within a host.
// Used to detect low-yield, duplicate-heavy, or spammy patterns
// and adjust crawl priority accordingly.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefixStats {
    // Hash of the host (scheme+host+port)
    pub host_key: Vec<u8>,
    // Hash of the prefix/template id
    pub prefix_key: Vec<u8>,
    // SimHash fingerprint of fetched content
    pub fp_minhash: Option<Vec<u64>>,
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

impl PrefixStats {
    pub async fn get_latest(
        session: Arc<DbSession>,
        host_key: Vec<u8>,
        prefix_key: Vec<u8>,
    ) -> Result<Self, AppError> {
        unimplemented!()
    }
}

impl Signal for PrefixStats {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS prefix_stats (
            host_key        blob,
            prefix_key      blob,
            fp_minhash      text,
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
            fp_minhash, last_update_ts,
            dup_page_ema, novelty_ema, near_dup_ema, variance_ema
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    async fn from_record(
        session: Arc<DbSession>,
        object_store: Arc<dyn ObjectStore>,
        base: SignalBase,
        record: Record,
    ) -> Result<Vec<Self>, AppError> {
        let url = Url::from_str(&record.uri)?;
        let host = extract_host(&url)?;
        let path = url.path().to_ascii_lowercase();
        let normalized_prefix = normalize_prefix(&path);
        let prefix_key = XXH3_128::hash(normalized_prefix.as_bytes())
            .to_be_bytes()
            .to_vec();
        let latest = Self::get_latest(session, base.host_key, base.prefix_key).await?;

        for m in record.metadata {
            let RecordMetadata::HttpResponse(resp) = m else {
                continue;
            };

            let duplicate = match (&latest.fp_minhash, &resp.minhash) {
                (Some(prev), Some(cur)) => jaccard_index(cur, prev) >= 0.95,
                _ => false,
            };
        }

        let dup_page_ema = update_dup_page_ema(
            prefix_stats.dup_page_ema,
            prefix_stats.last_update_ts,
            now_ts,
            duplicate,
            30.0 * 24.0 * 3600.0,
        );

        let result = Self {
            host_key: base.host_key,
            prefix_key: base.prefix_key,
            last_update_ts: Utc::now(),
        };

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
