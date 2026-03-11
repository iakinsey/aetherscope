use std::sync::Arc;

use async_trait::async_trait;
use cdrs_tokio::types::IntoRustByName;
use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};
use url::Url;
use xxhrs::XXH3_128;

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
    utils::web::extract_site,
};

fn update_ema(
    prev: f64,
    prev_ts: DateTime<Utc>,
    now: DateTime<Utc>,
    x: f64,
    tau_seconds: f64,
) -> f64 {
    let dt = (now - prev_ts).num_seconds().max(0) as f64;
    let decay = (-dt / tau_seconds).exp();
    prev * decay + x * (1.0 - decay)
}

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
impl InlinkAgg {
    async fn get_latest(
        session: Arc<DbSession>,
        target_key: Vec<u8>,
        kind: i8,
    ) -> Result<Self, AppError> {
        // If empty, emas are  0 and last_update_ts is now (probably can be passed in)
        const Q: &str = r#"
            SELECT
                target_key,
                kind,
                inlinks_ema,
                w_inlinks_ema,
                last_update_ts
            FROM inlink_agg
            WHERE target_key = ? 
            AND kind = ?;
        "#;

        let prepared = session.prepare(Q).await?;
        let result = session
            .exec_with_values(&prepared, query_values!(target_key.clone()))
            .await?;

        let row = match result.response_body()?.into_rows() {
            Some(mut rows) if !rows.is_empty() => rows.remove(0),
            _ => {
                return Ok(Self {
                    target_key,
                    kind,
                    inlinks_ema: 0.0,
                    w_inlinks_ema: 0.0,
                    last_update_ts: Utc::now(),
                });
            }
        };

        let inlinks_ema: Option<f64> = row.get_by_name("inlinks_ema")?;
        let w_inlinks_ema: Option<f64> = row.get_by_name("w_inlinks_ema")?;
        let last_update_ts: Option<DateTime<Utc>> = row.get_by_name("last_update_ts")?;

        Ok(Self {
            target_key,
            kind,
            inlinks_ema: inlinks_ema.unwrap_or(0.0),
            w_inlinks_ema: w_inlinks_ema.unwrap_or(0.0),
            last_update_ts: last_update_ts.unwrap_or_else(Utc::now),
        })
    }
}

#[async_trait]
impl Signal for InlinkAgg {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "inlink_agg"
    }

    fn create_table_query() -> &'static str
    where
        Self: Sized,
    {
        r#"
        CREATE TABLE IF NOT EXISTS inlink_agg (
            target_key     blob,
            kind           tinyint,
            inlinks_ema    double,
            w_inlinks_ema  double,
            last_update_ts timestamp,
            PRIMARY KEY ((target_key), kind)
        )
    "#
    }
    fn upsert_query() -> &'static str
    where
        Self: Sized,
    {
        r#"
        INSERT INTO inlink_agg (
            target_key, kind,
            inlinks_ema, w_inlinks_ema,
            last_update_ts
        ) VALUES (?, ?, ?, ?, ?)
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
        let mut now_ts: Option<DateTime<Utc>> = None;
        let mut out_uris: Vec<String> = Vec::new();

        for m in &record.metadata {
            match m {
                RecordMetadata::HttpResponse(resp) => {
                    if now_ts.is_none() {
                        now_ts = resp.timestamp;
                    }
                }
                RecordMetadata::Uris(u) => {
                    out_uris.extend(u.uris.iter().cloned());
                }
                _ => {}
            }
        }

        let now = match now_ts {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        let tau = 30.0 * 24.0 * 3600.0;

        let mut rows: Vec<Box<dyn Signal>> = Vec::new();

        for uri in out_uris {
            let url = match Url::parse(&uri) {
                Ok(u) => u,
                Err(_) => continue,
            };

            let host = match url.host_str() {
                Some(h) => h,
                None => continue,
            };

            let site = extract_site(&url)?;

            let url_key = XXH3_128::hash(uri.as_bytes()).to_be_bytes().to_vec();
            let host_key = XXH3_128::hash(host.as_bytes()).to_be_bytes().to_vec();
            let site_key = XXH3_128::hash(site.as_bytes()).to_be_bytes().to_vec();

            let targets = [(url_key, 0i8), (host_key, 1i8), (site_key, 2i8)];

            for (target_key, kind) in targets {
                let prev = Self::get_latest(session.clone(), target_key.clone(), kind).await?;
                let inlinks_ema = update_ema(prev.inlinks_ema, prev.last_update_ts, now, 1.0, tau);
                let w_inlinks_ema =
                    update_ema(prev.w_inlinks_ema, prev.last_update_ts, now, 1.0, tau);

                rows.push(Box::new(Self {
                    target_key,
                    kind,
                    inlinks_ema,
                    w_inlinks_ema,
                    last_update_ts: now,
                }));
            }
        }

        Ok(rows)
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
