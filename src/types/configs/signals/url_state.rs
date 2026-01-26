use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::types::traits::signal::Signal;

#[derive(Debug, Clone, PartialEq)]
pub struct UrlState {
    // Hash of the URL
    pub url_key: Vec<u8>,
    // Hash of the host (scheme+host+port)
    pub host_key: Vec<u8>,
    // Hash of eTLD+1 (or IP-literal)
    pub site_key: Vec<u8>,

    // Timestamp of the most recent fetch attempt (successful or failed)
    pub last_fetch_ts: DateTime<Utc>,
    // Last HTTP response status (or your synthetic status mapping for failures)
    pub last_status: i16,

    // Response metadata (may be absent depending on response / failure mode)
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,

    // Content-derived (may be absent: non-HTML, empty body, blocked, etc.)
    pub fp_simhash: Option<i64>,

    // Online signals (you can initialize these to 0.0 on first write if you prefer)
    pub change_ema: f64,
    pub soft404_ema: f64,
    pub thin_ema: f64,
    pub latency_ms_ema: f64,
    pub bytes_ema: f64,
}

impl Signal for UrlState {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS url_state (
            url_key         blob PRIMARY KEY,
            host_key        blob,
            site_key        blob,
            last_fetch_ts   timestamp,
            last_status     smallint,
            etag            text,
            last_modified   timestamp,
            fp_simhash      bigint,
            change_ema      double,
            soft404_ema     double,
            thin_ema        double,
            latency_ms_ema  double,
            bytes_ema       double
        )
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO url_state (
            url_key,
            host_key,
            site_key,
            last_fetch_ts,
            last_status,
            etag,
            last_modified,
            fp_simhash,
            change_ema,
            soft404_ema,
            thin_ema,
            latency_ms_ema,
            bytes_ema
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.url_key.clone(),
            self.host_key.clone(),
            self.site_key.clone(),
            self.last_fetch_ts.naive_utc(),
            self.last_status,
            self.etag.clone(),
            self.last_modified.map(|t| t.naive_utc()),
            self.fp_simhash,
            self.change_ema,
            self.soft404_ema,
            self.thin_ema,
            self.latency_ms_ema,
            self.bytes_ema
        )
    }
}
