use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::types::traits::signal::Signal;

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

impl Signal for HostStatsStripe {
    const CREATE_TABLE_QUERY: &'static str = r#"
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
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO host_stats_stripe (
            host_key, stripe,
            last_update_ts,
            latency_ms_ema, bytes_ema,
            http2xx_ema, http3xx_ema, http4xx_ema, http5xx_ema, http429_ema,
            timeout_ema,
            dup_outlink_ema, novel_outlink_ema,
            redirect_ema
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

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
