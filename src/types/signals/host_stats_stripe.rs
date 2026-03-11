use std::sync::Arc;

use async_trait::async_trait;
use cdrs_tokio::types::IntoRustByName;
use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};
use xxhash_rust::xxh3::xxh3_128;

use crate::types::{
    error::AppError,
    structs::{
        record::{Record, RecordMetadata},
        signal_base::SignalBase,
    },
    traits::{object_store::ObjectStore, signal::Signal},
};
use crate::utils::cassandra::DbSession;

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

// Striped per-host aggregate statistics.
// Host-level EMAs are spread across multiple stripes to avoid
// hot partitions; stripes are merged at read time.
#[derive(Debug, Clone, PartialEq)]
pub struct HostStatsStripe {
    // Hash of the host (scheme+host+port)
    pub host_key: Vec<u8>,
    // Stripe index for avoiding hot partitions
    pub stripe: i8,
    // Most recent update timestamp for this stripe
    pub last_update_ts: DateTime<Utc>,
    // EMA of latency
    pub latency_ms_ema: f64,
    // EMA of byte response size
    pub bytes_ema: f64,
    // EMA of 2xx responses
    pub http2xx_ema: f64,
    // EMA of 3xx responses
    pub http3xx_ema: f64,
    // EMA of 4xx responses
    pub http4xx_ema: f64,
    // EMA of 5xx responses
    pub http5xx_ema: f64,
    // EMA of 429 responses
    pub http429_ema: f64,
    // EMA of timeouts
    pub timeout_ema: f64,
    // EMA of duplicate outlinks
    pub dup_outlink_ema: f64,
    // EMA of novel outlinks
    pub novel_outlink_ema: f64,
    // EMA of redirects
    pub redirect_ema: f64,
}

impl HostStatsStripe {
    pub async fn get_latest(
        session: Arc<DbSession>,
        host_key: Vec<u8>,
        stripe: i8,
    ) -> Result<Self, AppError> {
        const Q: &str = r#"
            SELECT 
                host_key,
                stripe,
                last_update_ts,
                latency_ms_ema,
                bytes_ema,
                http2xx_ema,
                http3xx_ema,
                http4xx_ema,
                http5xx_ema,
                http429_ema,
                timeout_ema,
                dup_outlink_ema,
                novel_outlink_ema,
                redirect_ema
            FROM host_stats_stripe
            WHERE host_key = ?
            AND stripe = ?; 
        "#;

        let prepared = session.prepare(Q).await?;
        let result = session
            .exec_with_values(&prepared, query_values!(host_key.clone(), stripe.clone()))
            .await?;
        let row = match result.response_body()?.into_rows() {
            Some(mut rows) if !rows.is_empty() => rows.remove(0),
            _ => {
                return Ok(Self {
                    host_key,
                    stripe,
                    last_update_ts: Utc::now(),
                    latency_ms_ema: 0.0,
                    bytes_ema: 0.0,
                    http2xx_ema: 0.0,
                    http3xx_ema: 0.0,
                    http4xx_ema: 0.0,
                    http5xx_ema: 0.0,
                    http429_ema: 0.0,
                    timeout_ema: 0.0,
                    dup_outlink_ema: 0.0,
                    novel_outlink_ema: 0.0,
                    redirect_ema: 0.0,
                });
            }
        };

        let last_update_ts: Option<DateTime<Utc>> = row.get_by_name("last_update_ts")?;
        let latency_ms_ema: Option<f64> = row.get_by_name("latency_ms_ema")?;
        let bytes_ema: Option<f64> = row.get_by_name("bytes_ema")?;
        let http2xx_ema: Option<f64> = row.get_by_name("http2xx_ema")?;
        let http3xx_ema: Option<f64> = row.get_by_name("http3xx_ema")?;
        let http4xx_ema: Option<f64> = row.get_by_name("http4xx_ema")?;
        let http5xx_ema: Option<f64> = row.get_by_name("http5xx_ema")?;
        let http429_ema: Option<f64> = row.get_by_name("http429_ema")?;
        let timeout_ema: Option<f64> = row.get_by_name("timeout_ema")?;
        let dup_outlink_ema: Option<f64> = row.get_by_name("dup_outlink_ema")?;
        let novel_outlink_ema: Option<f64> = row.get_by_name("novel_outlink_ema")?;
        let redirect_ema: Option<f64> = row.get_by_name("redirect_ema")?;

        Ok(Self {
            host_key,
            stripe,
            last_update_ts: last_update_ts.unwrap_or_else(Utc::now),
            latency_ms_ema: latency_ms_ema.unwrap_or(0.0),
            bytes_ema: bytes_ema.unwrap_or(0.0),
            http2xx_ema: http2xx_ema.unwrap_or(0.0),
            http3xx_ema: http3xx_ema.unwrap_or(0.0),
            http4xx_ema: http4xx_ema.unwrap_or(0.0),
            http5xx_ema: http5xx_ema.unwrap_or(0.0),
            http429_ema: http429_ema.unwrap_or(0.0),
            timeout_ema: timeout_ema.unwrap_or(0.0),
            dup_outlink_ema: dup_outlink_ema.unwrap_or(0.0),
            novel_outlink_ema: novel_outlink_ema.unwrap_or(0.0),
            redirect_ema: redirect_ema.unwrap_or(0.0),
        })
    }
}

#[async_trait]
impl Signal for HostStatsStripe {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "host_stats_stripe"
    }

    fn create_table_query() -> &'static str
    where
        Self: Sized,
    {
        r#"
        CREATE TABLE IF NOT EXISTS host_stats_stripe (
            host_key          blob,
            stripe            tinyint,
            last_update_ts    timestamp,
            latency_ms_ema    double,
            bytes_ema         double,
            http2xx_ema       double,
            http3xx_ema       double,
            http4xx_ema       double,
            http5xx_ema       double,
            http429_ema       double,
            timeout_ema       double,
            dup_outlink_ema   double,
            novel_outlink_ema double,
            redirect_ema      double,
            PRIMARY KEY ((host_key), stripe)
        )
    "#
    }
    fn upsert_query() -> &'static str
    where
        Self: Sized,
    {
        r#"
        INSERT INTO host_stats_stripe (
            host_key, stripe,
            last_update_ts,
            latency_ms_ema, bytes_ema,
            http2xx_ema, http3xx_ema, http4xx_ema, http5xx_ema, http429_ema,
            timeout_ema,
            dup_outlink_ema, novel_outlink_ema,
            redirect_ema
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        const N_STRIPES: i8 = 16;
        let mut results: Vec<Box<dyn Signal>> = Vec::new();
        let host_key = base.host_key.clone();
        let stripe = ((xxh3_128(&host_key) as u64) % (N_STRIPES as u64)) as i8;

        for m in record.metadata {
            let RecordMetadata::HttpResponse(resp) = m else {
                continue;
            };

            let now = match resp.timestamp {
                Some(t) => t,
                None => continue,
            };

            let prev = Self::get_latest(session.clone(), host_key.clone(), stripe).await?;
            let tau_fast = 6.0 * 3600.0;
            let tau_medium = 7.0 * 24.0 * 3600.0;

            let latency_obs = resp
                .request
                .timestamp
                .signed_duration_since(now)
                .num_milliseconds()
                .abs() as f64;

            let latency_ms_ema = update_ema(
                prev.latency_ms_ema,
                prev.last_update_ts,
                now,
                latency_obs,
                tau_fast,
            );

            let bytes_obs = if let Some(v) = resp
                .response_headers
                .get("content-length")
                .and_then(|v| v.parse::<f64>().ok())
            {
                v
            } else if let Some(key) = &resp.key {
                object_store.get_size(key).await? as f64
            } else {
                0.0
            };

            let bytes_ema = update_ema(
                prev.bytes_ema,
                prev.last_update_ts,
                now,
                bytes_obs,
                tau_fast,
            );

            let status = resp.status.unwrap_or(0);

            let http2xx_ema = update_ema(
                prev.http2xx_ema,
                prev.last_update_ts,
                now,
                if (200..300).contains(&status) {
                    1.0
                } else {
                    0.0
                },
                tau_medium,
            );

            let http3xx_ema = update_ema(
                prev.http3xx_ema,
                prev.last_update_ts,
                now,
                if (300..400).contains(&status) {
                    1.0
                } else {
                    0.0
                },
                tau_medium,
            );

            let http4xx_ema = update_ema(
                prev.http4xx_ema,
                prev.last_update_ts,
                now,
                if (400..500).contains(&status) {
                    1.0
                } else {
                    0.0
                },
                tau_medium,
            );

            let http5xx_ema = update_ema(
                prev.http5xx_ema,
                prev.last_update_ts,
                now,
                if (500..600).contains(&status) {
                    1.0
                } else {
                    0.0
                },
                tau_medium,
            );

            let http429_ema = update_ema(
                prev.http429_ema,
                prev.last_update_ts,
                now,
                if status == 429 { 1.0 } else { 0.0 },
                tau_medium,
            );

            let timeout_ema = update_ema(
                prev.timeout_ema,
                prev.last_update_ts,
                now,
                if resp.error.is_some() { 1.0 } else { 0.0 },
                tau_medium,
            );

            let redirect_ema = update_ema(
                prev.redirect_ema,
                prev.last_update_ts,
                now,
                if (300..400).contains(&status) {
                    1.0
                } else {
                    0.0
                },
                tau_medium,
            );

            results.push(Box::new(HostStatsStripe {
                host_key: host_key.clone(),
                stripe,
                last_update_ts: now,
                latency_ms_ema,
                bytes_ema,
                http2xx_ema,
                http3xx_ema,
                http4xx_ema,
                http5xx_ema,
                http429_ema,
                timeout_ema,
                dup_outlink_ema: prev.dup_outlink_ema,
                novel_outlink_ema: prev.novel_outlink_ema,
                redirect_ema,
            }));
        }

        Ok(results)
    }

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.host_key.clone(),
            self.stripe,
            self.last_update_ts.naive_utc(),
            self.latency_ms_ema,
            self.bytes_ema,
            self.http2xx_ema,
            self.http3xx_ema,
            self.http4xx_ema,
            self.http5xx_ema,
            self.http429_ema,
            self.timeout_ema,
            self.dup_outlink_ema,
            self.novel_outlink_ema,
            self.redirect_ema
        )
    }
}
