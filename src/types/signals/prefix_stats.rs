use std::sync::Arc;

use async_trait::async_trait;
use cdrs_tokio::types::IntoRustByName;
use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::utils::cassandra::DbSession;
use crate::{
    types::{
        error::AppError,
        structs::{
            record::{Record, RecordMetadata},
            signal_base::SignalBase,
        },
        traits::{object_store::ObjectStore, signal::Signal},
    },
    utils::{
        cassandra::get_fp_minhash,
        hash::{jaccard_index, minhash_similarity},
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

fn update_novelty_ema(
    prev_ema: f64,
    prev_ts: DateTime<Utc>,
    now_ts: DateTime<Utc>,
    novel: bool,
    tau_seconds: f64,
) -> f64 {
    let dt = (now_ts - prev_ts).num_seconds().max(0) as f64;
    let decay = (-dt / tau_seconds).exp();

    let x = if novel { 1.0 } else { 0.0 };

    prev_ema * decay + x * (1.0 - decay)
}

fn update_near_dup_ema(
    prev_ema: f64,
    prev_ts: DateTime<Utc>,
    now_ts: DateTime<Utc>,
    near_dup: bool,
    tau_seconds: f64,
) -> f64 {
    let dt = (now_ts - prev_ts).num_seconds().max(0) as f64;
    let decay = (-dt / tau_seconds).exp();

    let x = if near_dup { 1.0 } else { 0.0 };

    prev_ema * decay + x * (1.0 - decay)
}

fn update_variance_ema(
    prev_ema: f64,
    prev_ts: DateTime<Utc>,
    now_ts: DateTime<Utc>,
    variance: f64,
    tau_seconds: f64,
) -> f64 {
    let dt = (now_ts - prev_ts).num_seconds().max(0) as f64;
    let decay = (-dt / tau_seconds).exp();

    prev_ema * decay + variance * (1.0 - decay)
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
    // EMA of content how often content changes
    pub variance_ema: f64,
}

impl PrefixStats {
    pub async fn get_latest(
        session: Arc<DbSession>,
        host_key: Vec<u8>,
        prefix_key: Vec<u8>,
    ) -> Result<Self, AppError> {
        const Q: &str = r#"
            SELECT
                host_key,
                prefix_key,
                fp_minhash,
                last_update_ts,
                dup_page_ema,
                novelty_ema,
                near_dup_ema,
                variance_ema
            FROM prefix_stats
            WHERE host_key = ? AND prefix_key = ?
        "#;

        let prepared = session.prepare(Q).await?;
        let result = session
            .exec_with_values(
                &prepared,
                query_values!(host_key.clone(), prefix_key.clone()),
            )
            .await?;

        let row = match result.response_body()?.into_rows() {
            Some(mut rows) if !rows.is_empty() => rows.remove(0),
            _ => {
                return Ok(Self {
                    host_key,
                    prefix_key,
                    fp_minhash: None,
                    last_update_ts: Utc::now(),
                    dup_page_ema: 0.0,
                    novelty_ema: 0.0,
                    near_dup_ema: 0.0,
                    variance_ema: 0.0,
                });
            }
        };

        let fp_minhash = get_fp_minhash(&row, "fp_minhash")?;
        let last_update_ts: Option<DateTime<Utc>> = row.get_by_name("last_update_ts")?;
        let dup_page_ema: Option<f64> = row.get_by_name("dup_page_ema")?;
        let novelty_ema: Option<f64> = row.get_by_name("novelty_ema")?;
        let near_dup_ema: Option<f64> = row.get_by_name("near_dup_ema")?;
        let variance_ema: Option<f64> = row.get_by_name("variance_ema")?;

        Ok(Self {
            host_key,
            prefix_key,
            fp_minhash,
            last_update_ts: last_update_ts.unwrap_or(Utc::now()),
            dup_page_ema: dup_page_ema.unwrap_or(0.0),
            novelty_ema: novelty_ema.unwrap_or(0.0),
            near_dup_ema: near_dup_ema.unwrap_or(0.0),
            variance_ema: variance_ema.unwrap_or(0.0),
        })
    }
}

#[async_trait]
impl Signal for PrefixStats {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "prefix_stats"
    }

    fn create_table_query() -> &'static str
    where
        Self: Sized,
    {
        r#"
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
    "#
    }
    fn upsert_query() -> &'static str
    where
        Self: Sized,
    {
        r#"
        INSERT INTO prefix_stats (
            host_key, prefix_key,
            fp_minhash, last_update_ts,
            dup_page_ema, novelty_ema, near_dup_ema, variance_ema
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#
    }
    async fn from_record(
        session: Arc<DbSession>,
        object_store: Arc<dyn ObjectStore>,
        base: SignalBase,
        record: Record,
    ) -> Result<Vec<Box<dyn Signal>>, AppError>
    where
        Self: Sized,
    {
        let latest =
            Self::get_latest(session, base.host_key.clone(), base.prefix_key.clone()).await?;
        let mut results: Vec<Box<dyn Signal>> = Vec::new();

        for m in record.metadata {
            let RecordMetadata::HttpResponse(resp) = m else {
                continue;
            };

            let duplicate = match (&latest.fp_minhash, &resp.minhash) {
                (Some(prev), Some(cur)) => jaccard_index(cur, prev) >= 0.95,
                _ => false,
            };

            let dup_page_ema = match resp.timestamp {
                Some(t) => update_dup_page_ema(
                    latest.dup_page_ema,
                    latest.last_update_ts,
                    t,
                    duplicate,
                    30.0 * 24.0 * 3600.0,
                ),
                None => latest.dup_page_ema,
            };

            let similarity = match (&latest.fp_minhash, &resp.minhash) {
                (Some(prev), Some(cur)) => minhash_similarity(cur, prev)?,
                _ => 0.0,
            };

            let novel = similarity < 0.85;
            let variance = 1.0 - similarity;

            let novelty_ema = match resp.timestamp {
                Some(t) => update_novelty_ema(
                    latest.novelty_ema,
                    latest.last_update_ts,
                    t,
                    novel,
                    21.0 * 24.0 * 3600.0,
                ),
                None => latest.novelty_ema,
            };
            let near_dup = similarity >= 0.85 && similarity < 0.95;

            let near_dup_ema = match resp.timestamp {
                Some(t) => update_near_dup_ema(
                    latest.near_dup_ema,
                    latest.last_update_ts,
                    t,
                    near_dup,
                    30.0 * 24.0 * 3600.0,
                ),
                None => latest.near_dup_ema,
            };

            let variance_ema = match resp.timestamp {
                Some(t) => update_variance_ema(
                    latest.variance_ema,
                    latest.last_update_ts,
                    t,
                    variance,
                    14.0 * 24.0 * 3600.0,
                ),
                None => latest.variance_ema,
            };

            let result = Self {
                host_key: base.host_key.clone(),
                prefix_key: base.prefix_key.clone(),
                fp_minhash: resp.minhash,
                last_update_ts: Utc::now(),
                dup_page_ema: dup_page_ema,
                novelty_ema,
                near_dup_ema,
                variance_ema,
            };

            results.push(Box::new(result));
        }

        Ok(results)
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
